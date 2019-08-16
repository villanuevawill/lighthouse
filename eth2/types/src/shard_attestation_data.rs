use crate::test_utils::TestRandom;
use crate::{Epoch, Hash256};

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::{CachedTreeHash, SignedRoot, TreeHash};

/// The data upon which an attestation is based.
///
/// Spec v0.6.3
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    Hash,
    Encode,
    Decode,
    TreeHash,
    CachedTreeHash,
    TestRandom,
    SignedRoot,
)]
pub struct ShardAttestationData {
    // LMD GHOST vote
    pub shard_block_root: Hash256,

    // Need to indicate which slot the attestation is for
    pub target_slot: Slot
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(ShardAttestationData);
    cached_tree_hash_tests!(ShardAttestationData);
}
