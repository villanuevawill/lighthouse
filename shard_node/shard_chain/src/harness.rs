use beacon_chain::{BeaconChain, BeaconChainTypes, BlockProcessingOutcome};
use crate::shard_chain::{ShardChain, ShardChainTypes, BlockProcessingOutcome as ShardBlockProcessingOutcome};
use lmd_ghost::LmdGhost;
use shard_lmd_ghost::{LmdGhost as ShardLmdGhost};
use slot_clock::{SlotClock, ShardSlotClock};
use slot_clock::{TestingSlotClock, ShardTestingSlotClock};
use state_processing::per_slot_processing;
use std::marker::PhantomData;
use std::sync::Arc;
use store::MemoryStore;
use shard_store::{MemoryStore as ShardMemoryStore};
use store::Store;
use shard_store::{Store as ShardStore};
use tree_hash::{SignedRoot, TreeHash};
use types::*;
use test_utils::TestingBeaconStateBuilder;
use crate::harness_tests;

// For now only accept 100% honest majority the entire time

/// Used to make the `Harness` generic over beacon types.
pub struct CommonBeaconTypes<L, E>
where
    L: LmdGhost<MemoryStore, E>,
    E: EthSpec,
{
    _phantom_l: PhantomData<L>,
    _phantom_e: PhantomData<E>,
}

/// Used to make the `Harness` generic over shard types.
pub struct CommonShardTypes<T, U>
where
    T: ShardLmdGhost<ShardMemoryStore, U>,
    U: ShardSpec,
{
    _phantom_t: PhantomData<T>,
    _phantom_u: PhantomData<U>,
}

impl<L, E> BeaconChainTypes for CommonBeaconTypes<L, E>
where
    L: LmdGhost<MemoryStore, E>,
    E: EthSpec,
{
    type Store = MemoryStore;
    type SlotClock = TestingSlotClock;
    type LmdGhost = L;
    type EthSpec = E;
}

impl<T, U> ShardChainTypes for CommonShardTypes<T, U>
where
    T: ShardLmdGhost<ShardMemoryStore, U>,
    U: ShardSpec,
{
    type Store = ShardMemoryStore;
    type SlotClock = ShardTestingSlotClock;
    type LmdGhost = T;
    type ShardSpec = U;
}

/// A testing harness which can instantiate a `BeaconChain` and `Shard Chain`, populating it with blocks and
/// attestations.
pub struct ShardChainHarness<L, E, T, U>
where
    L: LmdGhost<MemoryStore, E>,
    E: EthSpec,
    T: ShardLmdGhost<ShardMemoryStore, U>,
    U: ShardSpec,
{
    pub beacon_chain: Arc<BeaconChain<CommonBeaconTypes<L, E>>>,
    pub keypairs: Vec<Keypair>,
    pub beacon_spec: ChainSpec,
    pub shard_chain: ShardChain<CommonShardTypes<T, U>, CommonBeaconTypes<L, E>>,
    pub shard_spec: ChainSpec,
    _phantom_t: PhantomData<T>,
    _phantom_u: PhantomData<U>,
}

impl<L, E, T, U> ShardChainHarness<L, E, T, U>
where
    L: LmdGhost<MemoryStore, E>,
    E: EthSpec,
    T: ShardLmdGhost<ShardMemoryStore, U>,
    U: ShardSpec,
{
    /// Instantiate a new harness with `validator_count` initial validators.
    pub fn new(validator_count: usize) -> Self {
        let beacon_spec = E::default_spec();
        let shard_spec = U::default_spec();

        let beacon_store = Arc::new(MemoryStore::open());
        let shard_store = Arc::new(ShardMemoryStore::open());

        let beacon_state_builder =
            TestingBeaconStateBuilder::from_default_keypairs_file_if_exists(validator_count, &beacon_spec);
        let (beacon_genesis_state, keypairs) = beacon_state_builder.build();
        let shard_state = ShardState::genesis(&shard_spec, 0);

        let mut beacon_genesis_block = BeaconBlock::empty(&beacon_spec);
        beacon_genesis_block.state_root = Hash256::from_slice(&beacon_genesis_state.tree_hash_root());

        // Slot clock
        let beacon_slot_clock = TestingSlotClock::new(
            beacon_spec.genesis_slot,
            beacon_genesis_state.genesis_time,
            beacon_spec.seconds_per_slot,
        );

        let shard_slot_clock = ShardTestingSlotClock::new(
            ShardSlot::from(shard_spec.phase_1_fork_slot),
            beacon_genesis_state.genesis_time,
            shard_spec.shard_seconds_per_slot,
        );

        let beacon_chain = BeaconChain::from_genesis(
            beacon_store,
            beacon_slot_clock,
            beacon_genesis_state,
            beacon_genesis_block,
            beacon_spec.clone(),
        ).expect("Terminate if beacon chain generation fails");
        let beacon_chain_reference = Arc::new(beacon_chain);

        let shard_chain = ShardChain::from_genesis(
            shard_store,
            shard_slot_clock,
            shard_state,
            shard_spec.clone(),
            0,
            beacon_chain_reference.clone(),
        ).expect("Terminate if beacon chain generation fails");


        Self {
            beacon_chain: beacon_chain_reference.clone(),
            keypairs,
            beacon_spec,
            shard_chain: shard_chain,
            shard_spec,
            _phantom_t: PhantomData,
            _phantom_u: PhantomData,
        }
    }

    /// Advance slots of `BeaconChain` and `ShardChain`
    ///
    /// Does not produce blocks or attestations.
    pub fn advance_beacon_slot(&self) {
        self.beacon_chain.slot_clock.advance_slot();
        self.beacon_chain.catchup_state().expect("should catchup state");
    }

    pub fn advance_shard_slot(&self) {
        self.shard_chain.slot_clock.advance_slot();
        self.shard_chain.catchup_state().expect("should catchup state");
    }

    /// Extend the `BeaconChain` with some blocks and attestations. Returns the root of the
    /// last-produced block (the head of the chain).
    ///
    /// Chain will be extended by `num_blocks` blocks.
    ///
    /// The `block_strategy` dictates where the new blocks will be placed.
    ///
    /// The `attestation_strategy` dictates which validators will attest to the newly created
    /// blocks.
    pub fn extend_beacon_chain(
        &self,
        num_blocks: usize,
    ) -> Hash256 {
        let mut current_slot = self.beacon_chain.read_slot_clock().unwrap();
        let mut state = self.get_beacon_state_at_slot(current_slot - 1);
        let mut head_block_root = None;

        for _ in 0..num_blocks {
            while self.beacon_chain.read_slot_clock().expect("should have a slot") < current_slot {
                self.advance_beacon_slot();
            }

            let (block, new_state) = self.build_beacon_block(state.clone(), current_slot);

            let outcome = self
                .beacon_chain
                .process_block(block)
                .expect("should not error during block processing");

            if let BlockProcessingOutcome::Processed { block_root } = outcome {
                head_block_root = Some(block_root);

                self.add_beacon_attestations_to_op_pool(
                    &new_state,
                    block_root,
                    current_slot,
                );
            } else {
                panic!("block should be successfully processed: {:?}", outcome);
            }

            state = new_state;
            current_slot += 1;
        }

        head_block_root.expect("did not produce any blocks")
    }

    fn get_beacon_state_at_slot(&self, state_slot: Slot) -> BeaconState<E> {
        let state_root = self
            .beacon_chain
            .rev_iter_state_roots(self.beacon_chain.current_state().slot - 1)
            .find(|(_hash, slot)| *slot == state_slot)
            .map(|(hash, _slot)| hash)
            .expect("could not find state root");

        self.beacon_chain
            .store
            .get(&state_root)
            .expect("should read db")
            .expect("should find state root")
    }

    fn get_shard_state_at_slot(&self, state_slot: ShardSlot) -> ShardState<U> {
        let state_root = self
            .shard_chain
            .rev_iter_state_roots(self.shard_chain.current_state().slot - 1)
            .find(|(_hash, slot)| *slot == state_slot)
            .map(|(hash, _slot)| hash)
            .expect("could not find state root");

        self.shard_chain
            .store
            .get(&state_root)
            .expect("should read db")
            .expect("should find state root")
    }

    /// Returns a newly created block, signed by the proposer for the given slot.
    fn build_beacon_block(
        &self,
        mut state: BeaconState<E>,
        slot: Slot,
    ) -> (BeaconBlock, BeaconState<E>) {
        if slot < state.slot {
            panic!("produce slot cannot be prior to the state slot");
        }

        while state.slot < slot {
            per_slot_processing(&mut state, &self.beacon_spec)
                .expect("should be able to advance state to slot");
        }

        state.build_all_caches(&self.beacon_spec).unwrap();

        let proposer_index = self.beacon_chain
            .block_proposer(slot)
            .expect("should get block proposer from chain");

        let sk = &self.keypairs[proposer_index].sk;
        let fork = &state.fork.clone();

        let randao_reveal = {
            let epoch = slot.epoch(E::slots_per_epoch());
            let message = epoch.tree_hash_root();
            let domain = self.beacon_spec.get_domain(epoch, Domain::Randao, fork);
            Signature::new(&message, domain, sk)
        };

        let (mut block, state) = self
            .beacon_chain
            .produce_block_on_state(state, slot, randao_reveal)
            .expect("should produce block");

        block.signature = {
            let message = block.signed_root();
            let epoch = block.slot.epoch(E::slots_per_epoch());
            let domain = self.beacon_spec.get_domain(epoch, Domain::BeaconProposer, fork);
            Signature::new(&message, domain, sk)
        };

        (block, state)
    }

    /// Adds attestations to the `BeaconChain` operations pool to be included in future blocks.
    ///
    /// The `attestation_strategy` dictates which validators should attest.
    fn add_beacon_attestations_to_op_pool(
        &self,
        state: &BeaconState<E>,
        head_block_root: Hash256,
        head_block_slot: Slot,
    ) {
        let spec = &self.beacon_spec;
        let fork = &state.fork;

        let attesting_validators: Vec<usize> = (0..self.keypairs.len()).collect();

        state
            .get_crosslink_committees_at_slot(state.slot)
            .expect("should get committees")
            .iter()
            .for_each(|cc| {
                let committee_size = cc.committee.len();

                for (i, validator_index) in cc.committee.iter().enumerate() {
                     if attesting_validators.contains(validator_index) {
                        let data = self
                            .beacon_chain
                            .produce_attestation_data_for_block(
                                cc.shard,
                                head_block_root,
                                head_block_slot,
                                state,
                            )
                            .expect("should produce attestation data");

                        let mut aggregation_bitfield = Bitfield::new();
                        aggregation_bitfield.set(i, true);
                        aggregation_bitfield.set(committee_size, false);

                        let mut custody_bitfield = Bitfield::new();
                        custody_bitfield.set(committee_size, false);

                        let signature = {
                            let message = AttestationDataAndCustodyBit {
                                data: data.clone(),
                                custody_bit: false,
                            }
                            .tree_hash_root();

                            let domain =
                                spec.get_domain(data.target_epoch, Domain::Attestation, fork);

                            let mut agg_sig = AggregateSignature::new();
                            agg_sig.add(&Signature::new(
                                &message,
                                domain,
                                self.get_sk(*validator_index),
                            ));

                            agg_sig
                        };

                        let attestation = Attestation {
                            aggregation_bitfield,
                            data,
                            custody_bitfield,
                            signature,
                        };

                        self.beacon_chain
                            .process_attestation(attestation)
                            .expect("should process attestation");
                    }
                }
            });
    }

    /// Returns the secret key for the given validator index.
    fn get_sk(&self, validator_index: usize) -> &SecretKey {
        &self.keypairs[validator_index].sk
    }
}
