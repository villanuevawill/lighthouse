use crate::*;
use types::*;
use errors::Error;

pub mod errors;

pub fn per_shard_block_processing<T: ShardSpec, U: EthSpec>(
    state: &mut ShardState<T>,
    beacon_state: &BeaconState<U>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
    process_shard_block_header(state, beacon_state, block, spec);
    process_shard_attestations(state, beacon_state, block);
    process_shard_block_data_fees(state, beacon_state, block);
    Ok(())
}

pub fn process_shard_block_header<T: ShardSpec, U: EthSpec>(
    state: &mut ShardState<T>,
    beacon_state: &BeaconState<U>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
    verify!(block.slot == state.slot, ShardBlockProcessingError);
    verify!(block.parent_root == signing_root(state.latest_block_header), ShardBlockProcessingError);

    state.latest_block_header = block.block_header();

    let proposer_idx = get_shard_proposer_index(beacon_state, state.shard, block.slot);
    let pubkey = beacon_state.validator_registry[proposer_idx].pubkey;

    // perhaps the compute_epoch_of_shard_slot() function here is not correct, find the correct one
    let domain = get_domain(beacon_state, spec.domain_shard_proposer, compute_epoch_of_shard_slot(block.slot));
    let proposer = &state.validator_registry[proposer_idx];

    // update the error here at some point in the near future
    verify!(!proposer.slashed, ShardBlockProcessingError);

    verify_block_signature(&state, &beacon_state, &block, &spec);

    Ok(())
}

pub fn verify_block_signature<T: ShardSpec>(
    state: &ShardState<T>,
    beacon_state: &BeaconState<U>,
    block: &ShardBlock,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let block_proposer = &state.validator_registry
        [beacon_state.get_shard_proposer_index(block.slot, RelativeEpoch::Current, spec)?];

    let domain = spec.get_domain(
        block.slot.epoch(T::slots_per_epoch()),
        Domain::ShardProposer,
        &beacon_state.fork,
    );

    verify!(
        block 
            .signature 
            .verify(&block.signed_root()[..], domain, &block_proposer.pubkey)
    );

    Ok(())
}

pub fn process_shard_attestations<T: ShardSpec, U: EthSpec>(
    state: &mut ShardState<T>,
    beacon_state: &BeaconState<U>,
    attestations: &[Attestation],
    spec: &ChainSpec,
) -> Result<(), Error> {
    verify!(
        attestations.len() as u64 <= spec.max_attestations,
        BlockProcessingError
    );

    let shard_committee = beacon_state.get_shard_committee(state.current_epoch(), state.shard); 
    for (i, validator_idx) in shard_committee.iter().enumerate() {
        verify_block_signature(&state, &beacon_state, ) 
    }

    
}
