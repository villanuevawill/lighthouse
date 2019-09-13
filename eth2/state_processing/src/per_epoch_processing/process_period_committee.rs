use super::Error;
use tree_hash::TreeHash;
use types::*;

pub fn process_period_committee<T: EthSpec>(
    state: &mut BeaconState<T>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let current_epoch = state.current_epoch();

    if (current_epoch + 1) % spec.epochs_per_shard_period != 0 {
        return Ok(());
    }

    let shard_fork_period = ShardSlot::from(spec.phase_1_fork_slot)
        .epoch(spec.slots_per_epoch, spec.shard_slots_per_beacon_slot)
        .period(spec.epochs_per_shard_period);
    let current_period = current_epoch.period(spec.epochs_per_shard_period);

    if current_period - shard_fork_period + 2 >= 0 {
        state.advance_period_cache(spec);
        state.period_committee_roots[(current_period.as_u64() % spec.period_committee_root_length) as usize] =
            Hash256::from_slice(
                &state.period_caches[state.period_index(RelativePeriod::Next)]
                    .committees
                    .tree_hash_root()[..],
            );
    }

    Ok(())
}
