use crate::checkpoint::CheckPoint;
use crate::errors::{ShardChainError as Error, BlockProductionError};
use crate::fork_choice::{Error as ForkChoiceError, ForkChoice};
use lmd_ghost::LmdGhost;
// use operation_pool::{OperationPool};
use parking_lot::{RwLock, RwLockReadGuard};
use slot_clock::SlotClock;
use state_processing::per_block_processing::errors::{
    AttestationValidationError, AttesterSlashingValidationError,
};
use state_processing::{
    per_block_processing, per_block_processing_without_verifying_block_signature,
    per_slot_processing, BlockProcessingError,
};
use std::sync::Arc;
// use store::iter::{BestBlockRootsIterator, BlockIterator, BlockRootsIterator, StateRootsIterator};
use store::{Error as DBError, Store};
use tree_hash::TreeHash;
use types::*;

// Text included in blocks.
// Must be 32-bytes or panic.
//
//                          |-------must be this long------|
pub const GRAFFITI: &str = "sigp/lighthouse-0.0.0-prerelease";

pub trait ShardChainTypes {
    type Store: store::Store;
    type SlotClock: slot_clock::SlotClock;
    type LmdGhost: LmdGhost<Self::Store, Self::EthSpec>;
    type EthSpec: types::EthSpec;
}

/// Represents the "Shard Chain" component of Ethereum 2.0. It holds a reference to a parent Beacon Chain
pub struct ShardChain<T: ShardChainTypes, L: BeaconChainTypes> {
    pub parent_beacon: Arc<BeaconChain<L>>,
    pub shard: Shard,
    pub spec: ChainSpec,
    /// Persistent storage for blocks, states, etc. Typically an on-disk store, such as LevelDB.
    pub store: Arc<T::Store>,
    /// Reports the current slot, typically based upon the system clock.
    pub slot_clock: T::SlotClock,
    /// Stores all operations (e.g., transactions) that are candidates for
    /// inclusion in a block.
    pub op_pool: OperationPool<T::EthSpec>,
    /// Stores a "snapshot" of the chain at the time the head-of-the-chain block was recieved.
    canonical_head: RwLock<CheckPoint<T::EthSpec>>,
    /// The same state from `self.canonical_head`, but updated at the start of each slot with a
    /// skip slot if no block is recieved. This is effectively a cache that avoids repeating calls
    /// to `per_slot_processing`.
    state: RwLock<ShardState<T::EthSpec>>,
    /// The root of the genesis block.
    genesis_block_root: Hash256,
    /// A state-machine that is updated with information from the network and chooses a canonical
    /// head block.
    pub fork_choice: ForkChoice<T>,
}

impl<T: ShardChainTypes, L: BeaconChainTypes + ShardChainWrapper> ShardChain<T, L> {
    /// Instantiate a new Shard Chain, from genesis.
    pub fn from_genesis(
        store: Arc<T::Store>,
        slot_clock: T::SlotClock,
        mut genesis_state: ShardState<T::EthSpec>,
        genesis_block: ShardBlock,
        spec: ChainSpec,
        shard: Shard,
        parent_beacon: Arc<BeaconChain<L>>,
    ) -> Result<Self, Error> {
        genesis_state.build_all_caches(&spec)?;

        let state_root = genesis_state.canonical_root();
        store.put(&state_root, &genesis_state)?;

        let genesis_block_root = genesis_block.block_header().canonical_root();
        store.put(&genesis_block_root, &genesis_block)?;

        // Also store the genesis block under the `ZERO_HASH` key.
        let genesis_block_root = genesis_block.block_header().canonical_root();
        store.put(&spec.zero_hash, &genesis_block)?;

        let canonical_head = RwLock::new(CheckPoint::new(
            genesis_block.clone(),
            genesis_block_root,
            genesis_state.clone(),
            state_root,
        ));

        Ok(Self {
            parent_beacon,
            shard,
            spec,
            slot_clock,
            op_pool: OperationPool::new(),
            state: RwLock::new(genesis_state),
            canonical_head,
            genesis_block_root,
            fork_choice: ForkChoice::new(store.clone(), &genesis_block, genesis_block_root),
            store,
        })
    }

    /// Returns the beacon block body for each beacon block root in `roots`.
    ///
    /// Fails if any root in `roots` does not have a corresponding block.
    pub fn get_block_bodies(&self, roots: &[Hash256]) -> Result<Vec<ShardBlockBody>, Error> {
        let bodies: Result<Vec<ShardBlockBody>, _> = roots
            .iter()
            .map(|root| match self.get_block(root)? {
                Some(block) => Ok(block.body),
                None => Err(Error::DBInconsistent(format!("Missing block: {}", root))),
            })
            .collect();

        Ok(bodies?)
    }

    /// Returns the beacon block header for each beacon block root in `roots`.
    ///
    /// Fails if any root in `roots` does not have a corresponding block.
    pub fn get_block_headers(&self, roots: &[Hash256]) -> Result<Vec<ShardBlockHeader>, Error> {
        let headers: Result<Vec<ShardBlockHeader>, _> = roots
            .iter()
            .map(|root| match self.get_block(root)? {
                Some(block) => Ok(block.block_header()),
                None => Err(Error::DBInconsistent("Missing block".into())),
            })
            .collect();

        Ok(headers?)
    }
    /// Iterate in reverse (highest to lowest slot) through all blocks from the block at `slot`
    /// through to the genesis block.
    ///
    /// Returns `None` for headers prior to genesis or when there is an error reading from `Store`.
    ///
    /// Contains duplicate headers when skip slots are encountered.
    pub fn rev_iter_blocks(&self, slot: Slot) -> BlockIterator<T::EthSpec, T::Store> {
        BlockIterator::owned(self.store.clone(), self.state.read().clone(), slot)
    }

    /// Iterates in reverse (highest to lowest slot) through all block roots from `slot` through to
    /// genesis.
    ///
    /// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
    ///
    /// Contains duplicate roots when skip slots are encountered.
    pub fn rev_iter_block_roots(&self, slot: Slot) -> BlockRootsIterator<T::EthSpec, T::Store> {
        BlockRootsIterator::owned(self.store.clone(), self.state.read().clone(), slot)
    }

    /// Iterates in reverse (highest to lowest slot) through all block roots from largest
    /// `slot <= beacon_state.slot` through to genesis.
    ///
    /// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
    ///
    /// Contains duplicate roots when skip slots are encountered.
    pub fn rev_iter_best_block_roots(
        &self,
        slot: Slot,
    ) -> BestBlockRootsIterator<T::EthSpec, T::Store> {
        BestBlockRootsIterator::owned(self.store.clone(), self.state.read().clone(), slot)
    }

    /// Iterates in reverse (highest to lowest slot) through all state roots from `slot` through to
    /// genesis.
    ///
    /// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
    pub fn f(&self, slot: Slot) -> StateRootsIterator<T::EthSpec, T::Store> {
        StateRootsIterator::owned(self.store.clone(), self.state.read().clone(), slot)
    }

    /// Returns the block at the given root, if any.
    ///
    /// ## Errors
    ///
    /// May return a database error.
    pub fn get_block(&self, block_root: &Hash256) -> Result<Option<ShardBlock>, Error> {
        Ok(self.store.get(block_root)?)
    }

    /// Returns a read-lock guarded `BeaconState` which is the `canonical_head` that has been
    /// updated to match the current slot clock.
    pub fn current_state(&self) -> RwLockReadGuard<ShardState<T::EthSpec>> {
        self.state.read()
    }

    /// Returns a read-lock guarded `CheckPoint` struct for reading the head (as chosen by the
    /// fork-choice rule).
    ///
    /// It is important to note that the `beacon_state` returned may not match the present slot. It
    /// is the state as it was when the head block was received, which could be some slots prior to
    /// now.
    pub fn head(&self) -> RwLockReadGuard<CheckPoint<T::EthSpec>> {
        self.canonical_head.read()
    }

    /// Returns the slot of the highest block in the canonical chain.
    pub fn best_slot(&self) -> Slot {
        self.canonical_head.read().shard_block.slot
    }

    /// Ensures the current canonical `BeaconState` has been transitioned to match the `slot_clock`.
    pub fn catchup_state(&self) -> Result<(), Error> {
        let spec = &self.spec;

        let present_slot = match self.slot_clock.present_slot() {
            Ok(Some(slot)) => slot,
            _ => return Err(Error::UnableToReadSlot),
        };

        if self.state.read().slot < present_slot {
            let mut state = self.state.write();

            // If required, transition the new state to the present slot.
            for _ in state.slot.as_u64()..present_slot.as_u64() {
                // per_slot_processing(&mut *state, spec)?;
                // logic here to manage everything... add this in
            }

            state.build_all_caches(spec)?;
        }

        Ok(())
    }

    /// Build all of the caches on the current state.
    ///
    /// Ideally this shouldn't be required, however we leave it here for testing.
    pub fn ensure_state_caches_are_built(&self) -> Result<(), Error> {
        self.state.write().build_all_caches(&self.spec)?;

        Ok(())
    }

    /// Returns the validator index (if any) for the given public key.
    ///
    /// Information is retrieved from the present `beacon_state.validator_registry`.
    pub fn validator_index(&self, pubkey: &PublicKey) -> Option<usize> {
        // reference directly to beacon chain parent
        // needs to make sure it is part of this particular shard
        for (i, validator) in self
            .parent_beacon
            .current_state()
            .validator_registry
            .iter()
            .enumerate()
        {
            if validator.pubkey == *pubkey {
                if self.parent_beacon.current_state().get_attestation_duties(i).shard = self.shard {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Reads the slot clock, returns `None` if the slot is unavailable.
    ///
    /// The slot might be unavailable due to an error with the system clock, or if the present time
    /// is before genesis (i.e., a negative slot).
    ///
    /// This is distinct to `present_slot`, which simply reads the latest state. If a
    /// call to `read_slot_clock` results in a higher slot than a call to `present_slot`,
    /// `self.state` should undergo per slot processing.
    pub fn read_slot_clock(&self) -> Option<Slot> {
        match self.slot_clock.present_slot() {
            Ok(Some(some_slot)) => Some(some_slot),
            Ok(None) => None,
            _ => None,
        }
    }

    /// Reads the slot clock (see `self.read_slot_clock()` and returns the number of slots since
    /// genesis.
    pub fn slots_since_genesis(&self) -> Option<SlotHeight> {
        let now = self.read_slot_clock()?;
        let genesis_slot = self.spec.genesis_slot;

        if now < genesis_slot {
            None
        } else {
            Some(SlotHeight::from(now.as_u64() - genesis_slot.as_u64()))
        }
    }

    /// Returns slot of the present state.
    ///
    /// This is distinct to `read_slot_clock`, which reads from the actual system clock. If
    /// `self.state` has not been transitioned it is possible for the system clock to be on a
    /// different slot to what is returned from this call.
    pub fn present_slot(&self) -> Slot {
        self.state.read().slot
    }

    /// Returns the block proposer for a given slot.
    ///
    /// Information is read from the present `beacon_state` shuffling, only information from the
    /// present epoch is available.
    pub fn block_proposer(&self, slot: Slot, shard: Shard) -> Result<usize, Error> {
        // Update to go to beacon chain for this information
        // Ensures that the present state has been advanced to the present slot, skipping slots if
        // blocks are not present.
        // self.catchup_state()?;

        // // TODO: permit lookups of the proposer at any slot.
        let index = self.parent_beacon.get_shard_proposer_index(
            slot,
            shard,
            &self.spec,
        )?;

        Ok(index)
    }

    /// Produce an `AttestationData` that is valid for the present `slot` and given `shard`.
    ///
    /// Attests to the canonical chain.
    pub fn produce_attestation_data(&self) -> Result<ShardAttestationData, Error> {
        let state = self.state.read();
        let head_block_root = self.head().shard_block_root;
        let head_block_slot = self.head().shard_block.slot;

        self.produce_attestation_data_for_block(head_block_root, head_block_slot, &*state)
    }

    /// Produce an `AttestationData` that attests to the chain denoted by `block_root` and `state`.
    ///
    /// Permits attesting to any arbitrary chain. Generally, the `produce_attestation_data`
    /// function should be used as it attests to the canonical chain.
    pub fn produce_attestation_data_for_block(
        &self,
        head_block_root: Hash256,
        head_block_slot: Slot,
        state: &ShardState<T::EthSpec>,
    ) -> Result<ShardAttestationData, Error> {

        Ok(AttestationData {
            shard_block_root: head_block_root,
        })
    }

    /// Accept a new attestation from the network.
    ///
    /// If valid, the attestation is added to the `op_pool` and aggregated with another attestation
    /// if possible.
    pub fn process_attestation(
        &self,
        attestation: Attestation,
    ) -> Result<(), AttestationValidationError> {
        let result = self
            .op_pool
            .insert_attestation(attestation, &*self.state.read(), &self.spec);

        result
    }

    // This needs to be written and implemented
    pub fn process_transactions() -> {}

    /// Accept some block and attempt to add it to block DAG.
    ///
    /// Will accept blocks from prior slots, however it will reject any block from a future slot.
    pub fn process_block(&self, block: ShardBlock) -> Result<BlockProcessingOutcome, Error> {
        // In the future... need some logic here that will actually check to see
        // if the slot has been part of a finalized crosslink on the beacon chain
        // extra logic needed, but for our testnet/system we won't need this to the full degree
        // let finalized_slot = self
        //     .state
        //     .read()
        //     .finalized_epoch
        //     .start_slot(T::EthSpec::slots_per_epoch());

        // if block.slot <= finalized_slot {
        //     return Ok(BlockProcessingOutcome::FinalizedSlot);
        // }

        if block.slot == 0 {
            return Ok(BlockProcessingOutcome::GenesisBlock);
        }

        let block_root = block.block_header().canonical_root();

        if block_root == self.genesis_block_root {
            return Ok(BlockProcessingOutcome::GenesisBlock);
        }

        let present_slot = self
            .read_slot_clock()
            .ok_or_else(|| Error::UnableToReadSlot)?;

        if block.slot > present_slot {
            return Ok(BlockProcessingOutcome::FutureSlot {
                present_slot,
                block_slot: block.slot,
            });
        }

        if self.store.exists::<ShardBlock>(&block_root)? {
            return Ok(BlockProcessingOutcome::BlockIsAlreadyKnown);
        }

        // Load the blocks parent block from the database, returning invalid if that block is not
        // found.
        let parent_block_root = block.previous_block_root;
        let parent_block: ShardBlock = match self.store.get(&parent_block_root)? {
            Some(previous_block_root) => previous_block_root,
            None => {
                return Ok(BlockProcessingOutcome::ParentUnknown {
                    parent: parent_block_root,
                });
            }
        };

        // Load the parent blocks state from the database, returning an error if it is not found.
        // It is an error because if know the parent block we should also know the parent state.
        let parent_state_root = parent_block.state_root;
        let parent_state = self
            .store
            .get(&parent_state_root)?
            .ok_or_else(|| Error::DBInconsistent(format!("Missing state {}", parent_state_root)))?;

        // Transition the parent state to the block slot.
        let mut state: ShardState<T::EthSpec> = parent_state;
        for _ in state.slot.as_u64()..block.slot.as_u64() {
            per_slot_processing(&mut state, &self.spec)?;
        }

        // Apply the received block to its parent state (which has been transitioned into this
        // slot).
        match per_block_processing(&mut state, &block, &self.spec) {
            Err(BlockProcessingError::ShardStateError(e)) => {
                return Err(Error::ShardStateError(e))
            }
            Err(e) => return Ok(BlockProcessingOutcome::PerBlockProcessingError(e)),
            _ => {}
        }

        let state_root = state.canonical_root();

        if block.state_root != state_root {
            return Ok(BlockProcessingOutcome::StateRootMismatch);
        }

        // Store the block and state.
        self.store.put(&block_root, &block)?;
        self.store.put(&state_root, &state)?;
        

        // Register the new block with the fork choice service.
        self.fork_choice.process_block(&state, &block, block_root)?;

        // Execute the fork choice algorithm, enthroning a new head if discovered.
        //
        // Note: in the future we may choose to run fork-choice less often, potentially based upon
        // some heuristic around number of attestations seen for the block.
        self.fork_choice()?;
        Ok(BlockProcessingOutcome::Processed { block_root })
    }

    /// Produce a new block at the present slot.
    ///
    /// The produced block will not be inherently valid, it must be signed by a block producer.
    /// Block signing is out of the scope of this function and should be done by a separate program.
    pub fn produce_block(
        &self,
    ) -> Result<(ShardBlock, ShardState<T::EthSpec>), BlockProductionError> {
        let state = self.state.read().clone();
        let slot = self
            .read_slot_clock()
            .ok_or_else(|| BlockProductionError::UnableToReadSlot)?;

        self.produce_block_on_state(state, slot)
    }

    /// Produce a block for some `slot` upon the given `state`.
    ///
    /// Typically the `self.produce_block()` function should be used, instead of calling this
    /// function directly. This function is useful for purposefully creating forks or blocks at
    /// non-current slots.
    ///
    /// The given state will be advanced to the given `produce_at_slot`, then a block will be
    /// produced at that slot height.
    pub fn produce_block_on_state(
        &self,
        mut state: ShardState<T::EthSpec>,
        produce_at_slot: Slot,
    ) -> Result<(ShardBlock, ShardState<T::EthSpec>), BlockProductionError> {
        // If required, transition the new state to the present slot.
        while state.slot < produce_at_slot {
            per_slot_processing(&mut state, &self.spec)?;
        }

        let previous_block_root = if state.slot > 0 {
            *state
                .get_block_root(state.slot - 1)
                .map_err(|_| BlockProductionError::UnableToGetBlockRootFromState)?
        } else {
            state.latest_block_header.canonical_root()
        };

        let mut graffiti: [u8; 32] = [0; 32];
        graffiti.copy_from_slice(GRAFFITI.as_bytes());

        let mut block = ShardBlock {
            slot: state.slot,
            previous_block_root,
            state_root: Hash256::zero(), // Updated after the state is calculated.
            signature: Signature::empty_signature(), // To be completed by a validator.
            // need to add the attestations here
            body: ShardBlockBody {
                graffiti,
                attestations: self.op_pool.get_attestations(&state, &self.spec),
            },
        };

        per_block_processing_without_verifying_block_signature(&mut state, &block, &self.spec)?;

        let state_root = state.canonical_root();

        block.state_root = state_root;

        Ok((block, state))
    }

    /// Execute the fork choice algorithm and enthrone the result as the canonical head.
    pub fn fork_choice(&self) -> Result<(), Error> {
        // Determine the root of the block that is the head of the chain.
        let shard_block_root = self.fork_choice.find_head(&self)?;

        // If a new head was chosen.
        if shard_block_root != self.head().shard_block_root {
            let shard_block: ShardBlock = self
                .store
                .get(&shard_block_root)?
                .ok_or_else(|| Error::MissingShardBlock(shard_block_root))?;

            let shard_state_root = shard_block.state_root;
            let shard_state: ShardState<T::EthSpec> = self
                .store
                .get(&shard_state_root)?
                .ok_or_else(|| Error::MissingShardState(shard_state_root))?;

            // Never revert back past a finalized epoch.
            // NEED logic to make sure the slot is not coming from an older slot
            // if new_finalized_epoch < old_finalized_epoch {
            //     Err(Error::RevertedFinalizedEpoch {
            //         previous_epoch: old_finalized_epoch,
            //         new_epoch: new_finalized_epoch,
            //     })
            // } else {
            self.update_canonical_head(CheckPoint {
                shard_block: shard_block,
                shard_block_root,
                shard_state,
                shard_state_root,
            })?;

            Ok(())
        } else {
            Ok(())
        }
    }

    /// Update the canonical head to `new_head`.
    fn update_canonical_head(&self, new_head: CheckPoint<T::EthSpec>) -> Result<(), Error> {
        // Update the checkpoint that stores the head of the chain at the time it received the
        // block.
        *self.canonical_head.write() = new_head;

        // Update the always-at-the-present-slot state we keep around for performance gains.
        *self.state.write() = {
            let mut state = self.canonical_head.read().shard_state.clone();

            let present_slot = match self.slot_clock.present_slot() {
                Ok(Some(slot)) => slot,
                _ => return Err(Error::UnableToReadSlot),
            };

            // If required, transition the new state to the present slot.
            for _ in state.slot.as_u64()..present_slot.as_u64() {
                per_slot_processing(&mut state, &self.spec)?;
            }

            state.build_all_caches(&self.spec)?;

            state
        };

        Ok(())
    }

    /// Called after `self` has had a new block finalized.
    ///
    /// Performs pruning and finality-based optimizations.
    fn after_finalization(
        &self,
        old_finalized_epoch: Epoch,
        finalized_block_root: Hash256,
    ) -> Result<(), Error> {
        // Need to build logic here to manage pruning for shard as well
        // let finalized_block = self
        //     .store
        //     .get::<BeaconBlock>(&finalized_block_root)?
        //     .ok_or_else(|| Error::MissingBeaconBlock(finalized_block_root))?;

        // let new_finalized_epoch = finalized_block.slot.epoch(T::EthSpec::slots_per_epoch());

        // if new_finalized_epoch < old_finalized_epoch {
        //     Err(Error::RevertedFinalizedEpoch {
        //         previous_epoch: old_finalized_epoch,
        //         new_epoch: new_finalized_epoch,
        //     })
        // } else {
        //     self.fork_choice
        //         .process_finalization(&finalized_block, finalized_block_root)?;

        //     Ok(())
        // }
    }

    /// Returns `true` if the given block root has not been processed.
    pub fn is_new_block_root(&self, shard_block_root: &Hash256) -> Result<bool, Error> {
        Ok(!self.store.exists::<ShardBlock>(shard_block_root)?)
    }
}

impl From<DBError> for Error {
    fn from(e: DBError) -> Error {
        Error::DBError(e)
    }
}

impl From<ForkChoiceError> for Error {
    fn from(e: ForkChoiceError) -> Error {
        Error::ForkChoiceError(e)
    }
}

impl From<BeaconStateError> for Error {
    fn from(e: BeaconStateError) -> Error {
        Error::BeaconStateError(e)
    }
}

