use int_to_bytes::int_to_bytes8;
use ssz::ssz_encode;
use ssz_derive::{Decode, Encode};
use types::{AttestationData, BeaconState, ChainSpec, Domain, Epoch, EthSpec};

/// Serialized `AttestationData` augmented with a domain to encode the fork info.
#[derive(PartialEq, Eq, Clone, Hash, Debug, PartialOrd, Ord, Encode, Decode)]
pub struct AttestationId {
    v: Vec<u8>,
}

/// Number of domain bytes that the end of an attestation ID is padded with.
const DOMAIN_BYTES_LEN: usize = 16;

impl AttestationId {
    pub fn from_data<T: EthSpec>(
        attestation: &AttestationData,
        state: &ShardState<T>,
        spec: &ChainSpec,
    ) -> Self {
        let mut bytes = ssz_encode(attestation);
        let epoch = attestation.target_epoch;
        bytes.extend_from_slice(&AttestationId::compute_domain_bytes(epoch, state, spec));
        AttestationId { v: bytes }
    }

    pub fn compute_domain_bytes<T: EthSpec>(
        epoch: Epoch,
        state: &ShardState<T>,
        spec: &ChainSpec,
        slot: Slot
    ) -> Vec<u8> {
        // also pseudocoded here
        int_to_bytes8(spec.get_domain(epoch, Domain::Attestation, &state.fork)) + int_to_bytes8(slot)
    }

    pub fn domain_bytes_match(&self, domain_bytes: &[u8]) -> bool {
        &self.v[self.v.len() - DOMAIN_BYTES_LEN..] == domain_bytes
    }
}
