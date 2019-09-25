use crate::common::convert_to_indexed;
use types::*;

pub fn validate_attestation<T: ShardSpec>(
    state: &ShardState<T>,
    attestation: &Attestation,
    spec: &ChainSpec,
) -> Result<(), Error> {
    // validate_attestation_parametric(state, attestation, spec, true, false);

    Ok(())
}

pub fn validate_attestation_parametric<T: ShardSpec>(
    state: &ShardState<T>,
    attestation: &Attestation,
    spec: &ChainSpec,
    verify_signature: bool,
    time_independent_only: bool,
) -> Result<(), Error> {
    // let attestation_slot = state.get_attestation_slot(&attestation.data)?;

    Ok(())
}
