use crate::harness::{
    ShardChainHarness, CommonBeaconTypes,
};
use lmd_ghost::ThreadSafeReducedTree;
use shard_lmd_ghost::{ThreadSafeReducedTree as ShardThreadSafeReducedTree};
use rand::Rng;
use store::{MemoryStore, Store};
use shard_store::{MemoryStore as ShardMemoryStore, Store as ShardStore};
use types::test_utils::{SeedableRng, TestRandom, XorShiftRng};
use types::{Deposit, EthSpec, Hash256, MinimalEthSpec, MinimalShardSpec, Slot};

pub const VALIDATOR_COUNT: usize = 24;

pub type TestBeaconForkChoice = ThreadSafeReducedTree<MemoryStore, MinimalEthSpec>;
pub type TestShardForkChoice = ShardThreadSafeReducedTree<ShardMemoryStore, MinimalShardSpec>;

fn get_harness(validator_count: usize) -> ShardChainHarness<TestBeaconForkChoice, MinimalEthSpec, TestShardForkChoice, MinimalShardSpec> {
    let harness = ShardChainHarness::new(validator_count);

    // Move past the zero slot.
    harness.advance_beacon_slot();

    harness
}

#[test]
fn finalizes_with_full_participation() {
    let num_blocks_produced = MinimalEthSpec::slots_per_epoch() * 5;
    let harness = get_harness(VALIDATOR_COUNT);

    harness.extend_beacon_chain(
        num_blocks_produced as usize,
    );

    let state = &harness.beacon_chain.head().beacon_state;

    assert_eq!(
        state.slot, num_blocks_produced,
        "head should be at the current slot"
    );
    assert_eq!(
        state.current_epoch(),
        num_blocks_produced / MinimalEthSpec::slots_per_epoch(),
        "head should be at the expected epoch"
    );
    assert_eq!(
        state.current_justified_epoch,
        state.current_epoch() - 1,
        "the head should be justified one behind the current epoch"
    );
    assert_eq!(
        state.finalized_epoch,
        state.current_epoch() - 2,
        "the head should be finalized two behind the current epoch"
    );
}
