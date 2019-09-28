use lmd_ghost::ThreadSafeReducedTree;
use rest_api::{start_server, ApiConfig};
use shard_chain::ShardChainHarness;
use shard_lmd_ghost::ThreadSafeReducedTree as ShardThreadSafeReducedTree;
use shard_store::MemoryStore as ShardMemoryStore;
use slog::info;
use store::MemoryStore;
use tokio::prelude::*;
use tokio::runtime::TaskExecutor;
use tokio::timer::Interval;
use types::{EthSpec, MinimalEthSpec, MinimalShardSpec};

use std::time::{Duration, Instant};

pub const VALIDATOR_COUNT: usize = 24;

pub type TestBeaconForkChoice = ThreadSafeReducedTree<MemoryStore, MinimalEthSpec>;
pub type TestShardForkChoice = ShardThreadSafeReducedTree<ShardMemoryStore, MinimalShardSpec>;

pub fn run_shard_chain(log: &slog::Logger, executor: &TaskExecutor) -> () {
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

    let harness = get_harness(VALIDATOR_COUNT, log.clone());
    let fork_epoch = harness.beacon_spec.phase_1_fork_epoch;
    let num_blocks_produced = MinimalEthSpec::slots_per_epoch() * fork_epoch;

    info!(
        log,
        "Fast forwarding beacon node to phase 1 fork epoch";
        "fork_epoch" => format!("{:?}", fork_epoch),
    );

    harness.extend_beacon_chain((num_blocks_produced) as usize);

    info!(
        log,
        "Beacon chain successfully progressed to phase 1 fork epoch";
    );

    extend_shard_chain(log, &harness);

    let interval = Interval::new(Instant::now(), Duration::from_millis(3000));
    let shard_chain = harness.shard_chain.clone();
    let harness_logger = log.clone();
    let mut round = 0;

    executor.spawn(
        interval
            .for_each(move |_| {
                advance_shard_slot(&harness_logger, &harness);
                if round % 2 == 0 {
                    advance_beacon_slot(&harness_logger, &harness);
                }
                extend_shard_chain(&harness_logger, &harness);
                if round % 2 == 0 {
                    extend_beacon_chain(&harness_logger, &harness);
                }
                round = round + 1;
                Ok(())
            })
            .map_err(|e| panic!("interval errored; err={:?}", e)),
    );

    start_server(&ApiConfig::default(), &executor, shard_chain, &log);
}

fn get_harness(
    validator_count: usize,
    log: slog::Logger,
) -> ShardChainHarness<TestBeaconForkChoice, MinimalEthSpec, TestShardForkChoice, MinimalShardSpec>
{
    let harness = ShardChainHarness::new(validator_count, log);

    // Move past the zero slot
    harness.advance_beacon_slot();
    harness.advance_shard_slot();

    harness
}

fn advance_shard_slot(
    log: &slog::Logger,
    harness: &ShardChainHarness<
        TestBeaconForkChoice,
        MinimalEthSpec,
        TestShardForkChoice,
        MinimalShardSpec,
    >,
) -> () {
    harness.advance_shard_slot();
    info!(
        log,
        "Shard slot advanced";
        "slot" => format!("{:?}", harness.shard_chain.present_slot())
    );
}

fn advance_beacon_slot(
    log: &slog::Logger,
    harness: &ShardChainHarness<
        TestBeaconForkChoice,
        MinimalEthSpec,
        TestShardForkChoice,
        MinimalShardSpec,
    >,
) -> () {
    harness.advance_beacon_slot();
    let present_slot = harness.beacon_chain.present_slot();
    info!(
        log,
        "Beacon slot advanced";
        "slot" => format!("{:?}", present_slot)
    );

    if present_slot % MinimalEthSpec::slots_per_epoch() == 0 {
        info!(
            log,
            "Epoch Boundary";
            "Epoch" => format!("{:?}", present_slot.epoch(MinimalEthSpec::slots_per_epoch()))
        )
    }
}

fn extend_shard_chain(
    log: &slog::Logger,
    harness: &ShardChainHarness<
        TestBeaconForkChoice,
        MinimalEthSpec,
        TestShardForkChoice,
        MinimalShardSpec,
    >,
) -> () {
    harness.extend_shard_chain(1);

    if let Some(genesis_height) = harness.shard_chain.slots_since_genesis() {
        info!(
            log,
            "Shard Block Published";
            "best_slot" => harness.shard_chain.head().shard_block.slot,
            "latest_block_root" => format!("{}", harness.shard_chain.head().shard_block_root),
            "wall_clock_slot" => harness.shard_chain.read_slot_clock().unwrap(),
            "state_slot" => harness.shard_chain.head().shard_state.slot,
            "slots_since_genesis" => genesis_height,
        );
    }
}

fn extend_beacon_chain(
    log: &slog::Logger,
    harness: &ShardChainHarness<
        TestBeaconForkChoice,
        MinimalEthSpec,
        TestShardForkChoice,
        MinimalShardSpec,
    >,
) -> () {
    harness.extend_beacon_chain(1);

    if let Some(genesis_height) = harness.beacon_chain.slots_since_genesis() {
        info!(
            log,
            "Beacon Block Published";
            "best_slot" => harness.beacon_chain.head().beacon_block.slot,
            "latest_block_root" => format!("{}", harness.beacon_chain.head().beacon_block_root),
            "wall_clock_slot" => harness.beacon_chain.read_slot_clock().unwrap(),
            "state_slot" => harness.beacon_chain.head().beacon_state.slot,
            "slots_since_genesis" => genesis_height,
        );
    }
}
