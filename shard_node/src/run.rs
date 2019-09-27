use lmd_ghost::ThreadSafeReducedTree;
use rand::Rng;
use rest_api::{start_server, ApiConfig};
use shard_chain::ShardChainHarness;
use shard_lmd_ghost::ThreadSafeReducedTree as ShardThreadSafeReducedTree;
use shard_store::{MemoryStore as ShardMemoryStore, Store as ShardStore};
use slog::{error, info, warn};
use store::{MemoryStore, Store};
use tokio::prelude::*;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;
use tokio::runtime::TaskExecutor;
use tokio::timer::Interval;
use tokio_timer::clock::Clock;
use types::test_utils::{SeedableRng, TestRandom, XorShiftRng};
use types::{EthSpec, MinimalEthSpec, MinimalShardSpec, Slot};

use std::time::{Duration, Instant};

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

pub fn run_harness(log: &slog::Logger) -> () {
    // handle tokio result or error
    let runtime = Builder::new()
        .name_prefix("shard-")
        .clock(Clock::system())
        .build()
        .map_err(|e| format!("{:?}", e))
        .unwrap();

    let executor = runtime.executor();

    warn!(
        log,
        "This is a SIMULATION and is not safe to be used in any production setting."
    );

    info!(
        log,
        "Initializing beacon node";
        "validator count" => format!("{:?}", VALIDATOR_COUNT),
        "db_type" => "memory store",
    );

    info!(
        log,
        "Initializing shard node";
        "db_type" => "memory store",
        "shard_node_id" => "0",
    );

    let harness = get_harness(VALIDATOR_COUNT);
    let fork_epoch = harness.beacon_spec.phase_1_fork_epoch;
    let num_blocks_produced = MinimalEthSpec::slots_per_epoch() * fork_epoch;

    info!(
        log,
        "Fast forwarding beacon node to phase 1 fork epoch";
        "fork_epoch" => format!("{:?}", fork_epoch),
    );

    harness.extend_beacon_chain((num_blocks_produced + 1) as usize);
    harness.extend_shard_chain(1);

    let interval = Interval::new(Instant::now(), Duration::from_millis(3000));
    let mut test = 0;
    let mut active_logger = log.clone();

    let shard_chain = harness.shard_chain.clone();
    executor.spawn(
        interval
            .for_each(move |_| {
                harness.advance_shard_slot();
                info!(active_logger, "Shard Slot Advanced";);
                if test % 2 == 0 {
                    harness.advance_beacon_slot();
                    info!(active_logger, "Beacon Slot Advanced";);
                    harness.extend_beacon_chain(1);
                    info!(active_logger, "Beacon Block Produced";);
                }
                harness.extend_shard_chain(1);
                info!(active_logger, "Shard Block Produced";);
                test = test + 1;
                Ok(())
            })
            .map_err(|e| panic!("interval errored; err={:?}", e)),
    );

    start_server(&ApiConfig::default(), &executor, shard_chain, &log);

    // manage proper messages, etc
    runtime.shutdown_on_idle().wait().unwrap();
}
