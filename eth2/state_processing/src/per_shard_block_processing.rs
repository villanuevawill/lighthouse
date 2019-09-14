use types::*;

pub fn per_shard_block_processing<U: ShardSpec, T: EthSpec>(
    beacon_state: &BeaconState<T>,
    state: &mut ShardState<U>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
    Ok(())
}
