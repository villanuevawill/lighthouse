use crate::common::verify_bitfield_length;
use types::*;

/// Returns validator indices which participated in the attestation, sorted by increasing index.
///
/// Spec v0.6.3
pub fn get_attesting_indices<T: EthSpec>(
    state: &BeaconState<T>,
    attestation_data: &AttestationData,
    bitfield: &Bitfield,
) -> Result<Vec<usize>, BeaconStateError> {
    get_attesting_indices_unsorted(state, attestation_data, bitfield).map(|mut indices| {
        // Fast unstable sort is safe because validator indices are unique
        indices.sort_unstable();
        indices
    })
}

/// Returns validator indices which participated in the attestation, unsorted.
///
/// Spec v0.6.3
pub fn get_attesting_indices_unsorted<T: EthSpec>(
    state: &BeaconState<T>,
    attestation_data: &AttestationData,
    bitfield: &Bitfield,
) -> Result<Vec<usize>, BeaconStateError> {
    let target_relative_epoch =
        RelativeEpoch::from_epoch(state.current_epoch(), attestation_data.target_epoch)?;

    let committee =
        state.get_crosslink_committee_for_shard(attestation_data.shard, target_relative_epoch)?;

    if !verify_bitfield_length(&bitfield, committee.committee.len()) {
        return Err(BeaconStateError::InvalidBitfield);
    }

    Ok(committee
        .committee
        .iter()
        .enumerate()
        .filter_map(|(i, validator_index)| match bitfield.get(i) {
            Ok(true) => Some(*validator_index),
            _ => None,
        })
        .collect())
}


pub fn get_shard_attesting_indices<T: EthSpec>(
    shard: Shard,
    state: &BeaconState<T>,
    attestation_data: &ShardAttestationData,
    bitfield: &Bitfield,
) -> Result<Vec<usize>, BeaconStateError> {
    get_shard_attesting_indices_unsorted(shard, state, attestation_data, bitfield).map(|mut indices| {
        // Fast unstable sort is safe because validator indices are unique
        indices.sort_unstable();
        indices
    })
}

/// Returns validator indices which participated in the attestation, unsorted.
pub fn get_shard_attesting_indices_unsorted<T: EthSpec>(
    shard: Shard,
    state: &BeaconState<T>,
    attestation_data: &ShardAttestationData,
    bitfield: &Bitfield,
) -> Result<Vec<usize>, BeaconStateError> {
    let spec = T::default_spec();
    let target_epoch = attestation_data.target_slot.epoch(spec.slots_per_epoch, spec.shard_slots_per_beacon_slot);
    let committee =
        state.get_shard_committee(target_epoch, shard)?;

    if !verify_bitfield_length(&bitfield, committee.committee.len()) {
        return Err(BeaconStateError::InvalidBitfield);
    }

    Ok(committee
        .committee
        .iter()
        .enumerate()
        .filter_map(|(i, validator_index)| match bitfield.get(i) {
            Ok(true) => Some(*validator_index),
            _ => None,
        })
        .collect())
}
