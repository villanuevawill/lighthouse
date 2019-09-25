use crate::*;
use types::*;

pub fn get_shard_proposer_index<T:EthSpec>(
    beacon_state: &BeaconState<T>,
    shard: u64,
    epoch: Epoch,
) -> Result<u64, Error> {
    // let epoch = get_current_epoch(beacon_state);
    // let persistent_committee = get_period_committee()
}

pub fn get_persistent_committee<T: EthSpec>(
    beacon_state: &BeaconSTate<T>,
    shard: u64,
    epoch: Epoch,
) -> Result<(), Error> {
    Ok(())
}
