use crate::test_utils::TestRandom;
use crate::*;
use bls::Signature;

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::{SignedRoot, TreeHash};
use tree_hash_derive::{CachedTreeHash, SignedRoot, TreeHash};

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    CachedTreeHash,
    TestRandom,
    SignedRoot,
)]
pub struct ShardBlockHeader {
    pub slot: ShardSlot,
    pub shard: u64,
    pub parent_root: Hash256,
    pub beacon_block_root: Hash256,
    pub state_root: Hash256,
    // need to add body
    pub attestation: ShardAttestation,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

impl ShardBlockHeader {
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.signed_root()[..])
    }

    pub fn into_block(self) -> ShardBlock {
        // add body logic
        ShardBlock {
            shard: self.shard,
            slot: self.slot,
            beacon_block_root: self.beacon_block_root,
            parent_root: self.parent_root,
            state_root: self.state_root,
            attestation: self.attestation,
            signature: self.signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(ShardBlockHeader);
    cached_tree_hash_tests!(ShardBlockHeader);
}
