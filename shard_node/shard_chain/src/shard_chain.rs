use crate::checkpoint::CheckPoint;
use crate::errors::{ShardChainError as Error, BlockProductionError};
use beacon_chain::{BeaconChain, BeaconChainTypes};
use crate::fork_choice::{Error as ForkChoiceError, ForkChoice};
use shard_lmd_ghost::LmdGhost;
use shard_operation_pool::{OperationPool};
use parking_lot::{RwLock, RwLockReadGuard};
use slot_clock::SlotClock;
use state_processing::{
    per_shard_block_processing,
    per_shard_slot_processing, ShardBlockProcessingError,
};
use std::sync::Arc;
use store::{Store as BeaconStore, Error as BeaconDBError};
use shard_store::iter::{BestBlockRootsIterator, BlockIterator, BlockRootsIterator, StateRootsIterator};
use shard_store::{Error as DBError, Store};
use tree_hash::TreeHash;
use types::*;

// Text included in blocks.
// Must be 32-bytes or panic.
//
//                          |-------must be this long------|
pub const GRAFFITI: &str = "sigp/lighthouse-0.0.0-prerelease";

#[derive(Debug, PartialEq)]
pub enum BlockProcessingOutcome {
    /// Block was valid and imported into the block graph.
    Processed { block_root: Hash256 },
    /// The blocks parent_root is unknown.
    ParentUnknown { parent: Hash256 },
    /// The block slot is greater than the present slot.
    FutureSlot {
        present_slot: ShardSlot,
        block_slot: ShardSlot,
    },
    /// The block state_root does not match the generated state.
    StateRootMismatch,
    /// The block was a genesis block, these blocks cannot be re-imported.
    GenesisBlock,
    /// The slot is finalized, no need to import.
    FinalizedSlot,
    /// Block is already known, no need to re-import.
    BlockIsAlreadyKnown,
    /// The block could not be applied to the state, it is invalid.
    PerBlockProcessingError(ShardBlockProcessingError),
}

pub trait ShardChainTypes {
    type Store: shard_store::Store;
    type SlotClock: slot_clock::SlotClock;
    type LmdGhost: LmdGhost<Self::Store, Self::ShardSpec>;
    type ShardSpec: types::ShardSpec;
}

/// Represents the "Shard Chain" component of Ethereum 2.0. It holds a reference to a parent Beacon Chain
pub struct ShardChain<T: ShardChainTypes, L: BeaconChainTypes> {
    pub parent_beacon: Arc<BeaconChain<L>>,
    pub shard: Shard,
    pub spec: ChainSpec,
    pub store: Arc<T::Store>,
    pub slot_clock: T::SlotClock,
    pub op_pool: OperationPool<T::ShardSpec>,
    canonical_head: RwLock<CheckPoint<T::ShardSpec>>,
    state: RwLock<ShardState<T::ShardSpec>>,
    genesis_block_root: Hash256,
    pub crosslink_root: Hash256,
    pub fork_choice: ForkChoice<T>,
}

impl<T: ShardChainTypes, L: BeaconChainTypes> ShardChain<T, L> {
    pub fn from_genesis(
        store: Arc<T::Store>,
        slot_clock: T::SlotClock,
        mut genesis_state: ShardState<T::ShardSpec>,
        genesis_block: ShardBlock,
        spec: ChainSpec,
        shard: Shard,
        parent_beacon: Arc<BeaconChain<L>>,
    ) -> Result<Self, Error> {
        genesis_state.build_cache(&spec)?;

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
            crosslink_root: Hash256::default(),
            fork_choice: ForkChoice::new(store.clone(), &genesis_block, genesis_block_root),
            store,
        })
    }

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
    pub fn rev_iter_blocks(&self, slot: ShardSlot) -> BlockIterator<T::ShardSpec, T::Store> {
        BlockIterator::owned(self.store.clone(), self.state.read().clone(), slot)
    }

    /// Iterates in reverse (highest to lowest slot) through all block roots from `slot` through to
    /// genesis.
    ///
    /// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
    ///
    /// Contains duplicate roots when skip slots are encountered.
    pub fn rev_iter_block_roots(&self, slot: ShardSlot) -> BlockRootsIterator<T::ShardSpec, T::Store> {
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
        slot: ShardSlot,
    ) -> BestBlockRootsIterator<T::ShardSpec, T::Store> {
        BestBlockRootsIterator::owned(self.store.clone(), self.state.read().clone(), slot)
    }

    /// Iterates in reverse (highest to lowest slot) through all state roots from `slot` through to
    /// genesis.
    ///
    /// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
    pub fn rev_iter_state_roots(&self, slot: ShardSlot) -> StateRootsIterator<T::ShardSpec, T::Store> {
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

    /// Returns a read-lock guarded `ShardState` which is the `canonical_head` that has been
    /// updated to match the current slot clock.
    pub fn current_state(&self) -> RwLockReadGuard<ShardState<T::ShardSpec>> {
        self.state.read()
    }

    /// Returns a read-lock guarded `CheckPoint` struct for reading the head (as chosen by the
    /// fork-choice rule).
    ///
    /// It is important to note that the `shard_state` returned may not match the present slot. It
    /// is the state as it was when the head block was received, which could be some slots prior to
    /// now.
    pub fn head(&self) -> RwLockReadGuard<CheckPoint<T::ShardSpec>> {
        self.canonical_head.read()
    }

    /// Returns the slot of the highest block in the canonical chain.
    pub fn best_slot(&self) -> ShardSlot {
        self.canonical_head.read().shard_block.slot
    }

    /// Ensures the current canonical `ShardState` has been transitioned to match the `slot_clock`.
    pub fn catchup_state(&self) -> Result<(), Error> {
        let spec = &self.spec;

        let present_slot = match self.slot_clock.present_slot() {
            Ok(Some(slot)) => slot.shard_slot(spec.slots_per_epoch, spec.shard_slots_per_epoch),
            _ => return Err(Error::UnableToReadSlot),
        };

        if self.state.read().slot < present_slot {
            let mut state = self.state.write();

            // If required, transition the new state to the present slot.
            for _ in state.slot.as_u64()..present_slot.as_u64() {
                per_shard_slot_processing(&mut *state, spec)?;
            }

            state.build_cache(spec)?;
        }

        Ok(())
    }

    /// Build all of the caches on the current state.
    ///
    /// Ideally this shouldn't be required, however we leave it here for testing.
    pub fn ensure_state_caches_are_built(&self) -> Result<(), Error> {
        self.state.write().build_cache(&self.spec)?;

        Ok(())
    }

    /// Returns the validator index (if any) for the given public key.
    ///
    /// Information is retrieved from the present `beacon_state.validator_registry`.
    pub fn validator_index(&self, pubkey: &PublicKey) -> Option<usize> {
        // reference directly to beacon chain parent
        for (i, validator) in self
            .parent_beacon
            .head()
            .beacon_state
            .validator_registry
            .iter()
            .enumerate()
        {
            if validator.pubkey == *pubkey {
                return Some(i);
            }
        }
        None
    }

    /// Reads the slot clock and returns a ShardSlot, returns `None` if the slot is unavailable.
    ///
    /// The slot might be unavailable due to an error with the system clock, or if the present time
    /// is before genesis (i.e., a negative slot).
    ///
    /// This is distinct to `present_slot`, which simply reads the latest state. If a
    /// call to `read_slot_clock` results in a higher slot than a call to `present_slot`,
    /// `self.state` should undergo per slot processing.
    pub fn read_slot_clock(&self) -> Option<ShardSlot> {
        let spec = &self.spec;
        
        match self.slot_clock.present_slot() {
            Ok(Some(some_slot)) => Some(some_slot.shard_slot(spec.slots_per_epoch, spec.shard_slots_per_epoch)),
            Ok(None) => None,
            _ => None,
        }
    }

    /// Reads the slot clock (see `self.read_slot_clock()` and returns the number of slots since
    /// genesis.
    pub fn slots_since_genesis(&self) -> Option<ShardSlotHeight> {
        let now = self.read_slot_clock()?;
        let spec = &self.spec;
        let genesis_slot = spec.phase_1_fork_epoch * spec.shard_slots_per_epoch;


        if now < genesis_slot {
            None
        } else {
            Some(ShardSlotHeight::from(now.as_u64() - genesis_slot))
        }
    }

    /// Returns slot of the present state.
    ///
    /// This is distinct to `read_slot_clock`, which reads from the actual system clock. If
    /// `self.state` has not been transitioned it is possible for the system clock to be on a
    /// different slot to what is returned from this call.
    pub fn present_slot(&self) -> ShardSlot {
        self.state.read().slot
    }

    pub fn check_for_new_crosslink(mut self) -> Result<(), Error> {
        let beacon_state = self.parent_beacon.current_state();
        let crosslink_root = beacon_state.get_current_crosslink(self.shard)?.crosslink_data_root;
        let current_crossslink_root = self.crosslink_root;
        if crosslink_root != current_crossslink_root {
            self.crosslink_root = crosslink_root;
            self.after_crosslink(crosslink_root);
        }
        Ok(())
    }

    /// Returns the block proposer for a given slot.
    ///
    /// Information is read from the present `beacon_state`
    pub fn block_proposer(&self, slot: ShardSlot, shard: u64) -> Result<usize, Error> {
        // Ensures that the present state has been advanced to the present slot, skipping slots if
        // blocks are not present.
        self.catchup_state()?;

        let index = self.parent_beacon.current_state().get_shard_proposer_index(
            shard,
            slot,
        )?;

        Ok(index)
    }

    // /// Produce an `AttestationData` that is valid for the present `slot` and given `shard`.
    // ///
    // /// Attests to the canonical chain.
    // pub fn produce_attestation_data(&self) -> Result<ShardAttestationData, Error> {
    //     let state = self.state.read();
    //     let head_block_root = self.head().shard_block_root;
    //     let head_block_slot = self.head().shard_block.slot;

    //     self.produce_attestation_data_for_block(head_block_root, head_block_slot, &*state)
    // }

    // /// Produce an `AttestationData` that attests to the chain denoted by `block_root` and `state`.
    // ///
    // /// Permits attesting to any arbitrary chain. Generally, the `produce_attestation_data`
    // /// function should be used as it attests to the canonical chain.
    // pub fn produce_attestation_data_for_block(
    //     &self,
    //     head_block_root: Hash256,
    //     head_block_slot: Slot,
    //     state: &ShardState<T::EthSpec>,
    // ) -> Result<ShardAttestationData, Error> {

    //     Ok(AttestationData {
    //         shard_block_root: head_block_root,
    //     })
    // }

    /// Accept a new attestation from the network.
    ///
    /// If valid, the attestation is added to the `op_pool` and aggregated with another attestation
    /// if possible.
    pub fn process_attestation(
        &self,
        attestation: ShardAttestation,
    ) -> () {
        self.op_pool
            .insert_attestation(attestation, &self.parent_beacon.current_state(), &self.spec);
    }

    /// Accept some block and attempt to add it to block DAG.
    ///
    /// Will accept blocks from prior slots, however it will reject any block from a future slot.
    pub fn process_block(&self, block: ShardBlock) -> Result<BlockProcessingOutcome, Error> {
        let spec = &self.spec;
        let beacon_state = &self.parent_beacon.current_state();
        
        let finalized_slot = beacon_state
            .finalized_epoch
            .start_slot(spec.slots_per_epoch)
            .shard_slot(spec.slots_per_epoch, spec.shard_slots_per_epoch);

        if block.slot <= finalized_slot {
            return Ok(BlockProcessingOutcome::FinalizedSlot);
        }

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
        let parent_block_root = block.parent_root;
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
        let mut state: ShardState<T::ShardSpec> = parent_state;
        for _ in state.slot.as_u64()..block.slot.as_u64() {
            per_shard_slot_processing(&mut state, &self.spec)?;
        }

        // Apply the received block to its parent state (which has been transitioned into this
        // slot).
        match per_shard_block_processing(beacon_state, &mut state, &block, &self.spec) {
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
        self.fork_choice.process_block(&beacon_state, &block, block_root)?;

        // Execute the fork choice algorithm, enthroning a new head if discovered.
        //
        // Note: in the future we may choose to run fork-choice less often, potentially based upon
        // some heuristic around number of attestations seen for the block.
        self.fork_choice()?;
        Ok(BlockProcessingOutcome::Processed { block_root })
    }

    // /// Produce a new block at the present slot.
    // ///
    // /// The produced block will not be inherently valid, it must be signed by a block producer.
    // /// Block signing is out of the scope of this function and should be done by a separate program.
    // pub fn produce_block(
    //     &self,
    // ) -> Result<(ShardBlock, ShardState<T::EthSpec>), BlockProductionError> {
    //     let state = self.state.read().clone();
    //     let slot = self
    //         .read_slot_clock()
    //         .ok_or_else(|| BlockProductionError::UnableToReadSlot)?;

    //     self.produce_block_on_state(state, slot)
    // }

    // /// Produce a block for some `slot` upon the given `state`.
    // ///
    // /// Typically the `self.produce_block()` function should be used, instead of calling this
    // /// function directly. This function is useful for purposefully creating forks or blocks at
    // /// non-current slots.
    // ///
    // /// The given state will be advanced to the given `produce_at_slot`, then a block will be
    // /// produced at that slot height.
    // pub fn produce_block_on_state(
    //     &self,
    //     mut state: ShardState<T::EthSpec>,
    //     produce_at_slot: Slot,
    // ) -> Result<(ShardBlock, ShardState<T::EthSpec>), BlockProductionError> {
    //     // If required, transition the new state to the present slot.
    //     while state.slot < produce_at_slot {
    //         per_slot_processing(&mut state, &self.spec)?;
    //     }

    //     let previous_block_root = if state.slot > 0 {
    //         *state
    //             .get_block_root(state.slot - 1)
    //             .map_err(|_| BlockProductionError::UnableToGetBlockRootFromState)?
    //     } else {
    //         state.latest_block_header.canonical_root()
    //     };

    //     let mut graffiti: [u8; 32] = [0; 32];
    //     graffiti.copy_from_slice(GRAFFITI.as_bytes());

    //     let mut block = ShardBlock {
    //         slot: state.slot,
    //         previous_block_root,
    //         state_root: Hash256::zero(), // Updated after the state is calculated.
    //         signature: Signature::empty_signature(), // To be completed by a validator.
    //         // need to add the attestations here
    //         body: ShardBlockBody {
    //             graffiti,
    //             attestations: self.op_pool.get_attestations(&state, &self.spec),
    //         },
    //     };

    //     per_block_processing_without_verifying_block_signature(&mut state, &block, &self.spec)?;

    //     let state_root = state.canonical_root();

    //     block.state_root = state_root;

    //     Ok((block, state))
    // }

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
            let shard_state: ShardState<T::ShardSpec> = self
                .store
                .get(&shard_state_root)?
                .ok_or_else(|| Error::MissingShardState(shard_state_root))?;

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

    /// Execute the fork choice algorithm and enthrone the result as the canonical head.
    /// Update the canonical head to `new_head`.
    fn update_canonical_head(&self, new_head: CheckPoint<T::ShardSpec>) -> Result<(), Error> {
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
                per_shard_slot_processing(&mut state, &self.spec)?;
            }

            state.build_cache(&self.spec)?;

            state
        };

        Ok(())
    }

    /// Called after `self` has found a new crosslink
    ///
    /// Performs pruning and fork choice optimizations after recognized crosslinks.
    fn after_crosslink(&self, crosslink_root: Hash256) -> Result<(), Error> {
        let crosslink_block = self
            .store
            .get::<ShardBlock>(&crosslink_root)?
            .ok_or_else(|| Error::MissingShardBlock(crosslink_root))?;

        self.fork_choice
            .process_finalization(&crosslink_block, crosslink_root)?;

        Ok(())
    }

    // /// Returns `true` if the given block root has not been processed.
    // pub fn is_new_block_root(&self, shard_block_root: &Hash256) -> Result<bool, Error> {
    //     Ok(!self.store.exists::<ShardBlock>(shard_block_root)?)
    // }
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

impl From<ShardStateError> for Error {
    fn from(e: ShardStateError) -> Error {
        Error::ShardStateError(e)
    }
}
