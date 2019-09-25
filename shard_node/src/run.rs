use shard_chain::ShardChainHarness;
use lmd_ghost::ThreadSafeReducedTree;
use rand::Rng;
use shard_lmd_ghost::ThreadSafeReducedTree as ShardThreadSafeReducedTree;
use shard_store::{MemoryStore as ShardMemoryStore, Store as ShardStore};
use store::{MemoryStore, Store};
use types::test_utils::{SeedableRng, TestRandom, XorShiftRng};
use types::{EthSpec, MinimalEthSpec, MinimalShardSpec, Slot};

pub const VALIDATOR_COUNT: usize = 24;

pub type TestBeaconForkChoice = ThreadSafeReducedTree<MemoryStore, MinimalEthSpec>;
pub type TestShardForkChoice = ShardThreadSafeReducedTree<ShardMemoryStore, MinimalShardSpec>;

fn get_harness(
    validator_count: usize,
) -> ShardChainHarness<TestBeaconForkChoice, MinimalEthSpec, TestShardForkChoice, MinimalShardSpec>
{
    let harness = ShardChainHarness::new(validator_count);

    // Move past the zero slot
    harness.advance_beacon_slot();
    harness.advance_shard_slot();

    harness
}

pub fn run_harness() -> () {
    let harness = get_harness(VALIDATOR_COUNT);
    let num_blocks_produced =
        MinimalEthSpec::slots_per_epoch() * harness.beacon_spec.phase_1_fork_epoch;

    harness.extend_beacon_chain((num_blocks_produced + 1) as usize);
    harness.extend_shard_chain(1, vec![]);   
    println!("{:?}", harness.shard_chain.current_state().clone());
}
