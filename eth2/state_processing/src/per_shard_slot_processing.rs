use crate::*;
use types::*;

pub fn per_shard_slot_processing<T: EthSpec>(
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

    state.slot += 1;

    Ok(())
}
