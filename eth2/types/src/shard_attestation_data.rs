use crate::test_utils::TestRandom;
use crate::{Hash256, ShardSlot};

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Encode, Decode, TreeHash, TestRandom
)]
pub struct ShardAttestationData {
    // LMD GHOST vote
    pub shard_block_root: Hash256,

    // Need to indicate which slot the attestation is for
    pub target_slot: ShardSlot,
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(ShardAttestationData);
    cached_tree_hash_tests!(ShardAttestationData);
}
