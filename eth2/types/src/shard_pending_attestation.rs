use crate::test_utils::TestRandom;
use crate::{ShardAttestationData, Bitfield};

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::{CachedTreeHash, TreeHash};

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
    CachedTreeHash,
    TestRandom,
)]
pub struct ShardPendingAttestation {
    pub aggregation_bitfield: Bitfield,
    pub data: ShardAttestationData,
    pub inclusion_delay: u64,
    pub proposer_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(ShardPendingAttestation);
    cached_tree_hash_tests!(ShardPendingAttestation);
}
