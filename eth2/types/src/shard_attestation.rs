use super::{AggregateSignature, BitList, ShardAttestationData};
use crate::test_utils::TestRandom;

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::{SignedRoot, TreeHash};

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    SignedRoot,
)]
#[serde(bound = "T: ShardSpec")]
pub struct ShardAttestation<T: ShardSpec> {
    pub aggregation_bitfield: BitList<T::ShardCommitteeTargetSize>,
    pub data: ShardAttestationData,
    #[signed_root(skip_hashing)]
    pub signature: AggregateSignature,
}

impl<T: ShardSpec>ShardAttestation<T> {
    /// Are the aggregation bitfields of these attestations disjoint?
    pub fn signers_disjoint_from(&self, other: &Self) -> bool {
        self.aggregation_bits
            .intersection(&other.aggregation_bits)
            .is_zero()
    }

    /// Aggregate another Attestation into this one.
    ///
    /// The aggregation bitfields must be disjoint, and the data must be the same.
    pub fn aggregate(&mut self, other: &Self) {
        debug_assert_eq!(self.data, other.data);
        debug_assert!(self.signers_disjoint_from(other));

        self.aggregation_bits = self.aggregation_bits.union(&other.aggregation_bits);
        self.signature.add_aggregate(&other.signature);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(ShardAttestation<MainnetShardSpec>);
}
