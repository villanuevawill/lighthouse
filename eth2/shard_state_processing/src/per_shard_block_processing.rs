use crate::*;
use types::*;
use errors::{
    Error
};

pub mod errors;

pub fn per_shard_block_processing<T: EthSpec, U: ShardSpec>(
    beacon_state: &BeaconState<T>,
    state: &mut ShardState<U>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
    process_shard_block_header(beacon_state, state, block, spec);
    // process_shard_attestations
    // process_shard_block_body
    Ok(())
}

pub fn process_shard_block_header<T: EthSpec, U: ShardSpec>(
    beacon_state: &BeaconState<T>,
    state: &mut ShardState<U>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
//     verify!(block.slot == state.slot, Error::BlockProcessingError);
    // NOTE: synonymous to the temporary_block_header in beacon_state processing
    state.latest_block_header = block.block_header();
    // needs to be ShardBlockHeader.core
    
    Ok(())
}
