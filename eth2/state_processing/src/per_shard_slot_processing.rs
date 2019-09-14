use crate::*;
use types::*;
use tree_hash::TreeHash;

pub fn per_shard_slot_processing<T: ShardSpec>(
    state: &mut ShardState<T>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    if (state
        .slot
        .epoch(spec.slots_per_epoch, spec.shard_slots_per_beacon_slot)
        + 1)
        % spec.epochs_per_shard_period
        == 0
    {
        // include period processing here :)
    }

    process_shard_slot(state, spec);

    state.slot += 1;

    Ok(())
}

// need to put this in separate directory (process slots)
fn process_shard_slot<T: ShardSpec>(
    state: &mut ShardState<T>,
    spec: &ChainSpec,
) -> () {
    let previous_state_root = Hash256::from_slice(&state.tree_hash_root()[..]);

    if state.latest_block_header.state_root == spec.zero_hash {
        state.latest_block_header.state_root = previous_state_root;
    }

    let mut depth = 0;
    while (state.slot.as_u64() % u64::pow(2, depth as u32) == 0 as u64) && (depth < T::history_accumulator_depth() as u64) {
        state.history_accumulator[depth as usize] = previous_state_root;
        depth += 1;
    }
}
