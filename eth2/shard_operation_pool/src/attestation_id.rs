use int_to_bytes::int_to_bytes8;
use ssz::ssz_encode;
use ssz_derive::{Decode, Encode};
use types::{BeaconState, ChainSpec, Domain, Epoch, EthSpec, ShardAttestationData, ShardSlot};

/// Serialized `AttestationData` augmented with a domain to encode the fork info.
#[derive(PartialEq, Eq, Clone, Hash, Debug, PartialOrd, Ord, Encode, Decode)]
pub struct AttestationId {
    v: Vec<u8>,
}

/// Number of domain bytes that the end of an attestation ID is padded with.
const DOMAIN_BYTES_LEN: usize = 16;

impl AttestationId {
    pub fn from_data<T: EthSpec>(
        attestation: &ShardAttestationData,
        beacon_state: &BeaconState<T>,
        spec: &ChainSpec,
    ) -> Self {
        let mut bytes = ssz_encode(attestation);
        let slot = attestation.target_slot;
        let epoch = slot.epoch(spec.slots_per_epoch, spec.shard_slots_per_beacon_slot);
        bytes.extend_from_slice(&AttestationId::compute_domain_bytes(
            epoch,
            slot,
            beacon_state,
            spec,
        ));
        AttestationId { v: bytes }
    }

    pub fn compute_domain_bytes<T: EthSpec>(
        epoch: Epoch,
        slot: ShardSlot,
        beacon_state: &BeaconState<T>,
        spec: &ChainSpec,
    ) -> Vec<u8> {
        let mut domain_bytes =
            int_to_bytes8(spec.get_domain(epoch, Domain::Attestation, &beacon_state.fork));
        let mut slot_identifying_bytes = int_to_bytes8(slot.into());

        domain_bytes.append(&mut slot_identifying_bytes);
        domain_bytes
    }

    pub fn domain_bytes_match(&self, domain_bytes: &[u8]) -> bool {
        &self.v[self.v.len() - DOMAIN_BYTES_LEN..] == domain_bytes
    }
}
