use types::*;

pub fn per_shard_block_processing<T: EthSpec>(
    beacon_state: &BeaconState<T>,
    state: &mut ShardState<T>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
    Ok(())
}
