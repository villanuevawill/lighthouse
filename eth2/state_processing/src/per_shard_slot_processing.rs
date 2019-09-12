use crate::*;
use types::*;

pub fn per_shard_slot_processing<T: EthSpec>(
    state: &mut ShardState<T>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    // period_processing logic
    state.slot += 1;

    Ok(())
}
