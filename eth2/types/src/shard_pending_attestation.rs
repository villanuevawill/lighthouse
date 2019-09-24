use crate::test_utils::TestRandom;
use crate::{BitList, ShardAttestationData};

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// An attestation that has been included in the state but not yet fully processed.
///
/// Spec v0.6.3
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct ShardPendingAttestation<T: ShardSpec> {
    pub aggregation_bitfield:  BitList<T::ShardCommitteeTargetSize>,
    pub data: ShardAttestationData,
    pub proposer_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(ShardPendingAttestation);
    cached_tree_hash_tests!(ShardPendingAttestation);
}
