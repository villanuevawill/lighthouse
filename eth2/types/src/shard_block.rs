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
pub struct ShardBlock {
    pub shard: u64,
    pub slot: ShardSlot,
    pub previous_block_root: Hash256,
    pub state_root: Hash256,
    pub attestation: ShardAttestation,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

impl ShardBlock {
    pub fn empty(spec: &ChainSpec, shard: u64) -> BeaconBlock {
        ShardBlock {
            shard,
            slot: spec.phase_1_fork_slot as ShardSlot,
            previous_block_root: spec.zero_hash,
            state_root: spec.zero_hash,
            attestation: ShardAttestation::default(),
            signature: Signature::empty_signature(),
        }
    }

    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.signed_root()[..])
    }

    pub fn block_header(&self) -> ShardBlockHeader {
        ShardBlockHeader {
            shard: self.shard,
            slot: self.slot,
            previous_block_root: self.previous_block_root,
            state_root: self.state_root,
            block_body_root: Hash256::from_slice(&self.body.tree_hash_root()[..]),
            signature: self.signature.clone(),
        }
    }

    pub fn temporary_block_header(&self, spec: &ChainSpec) -> ShardBlockHeader {
        ShardBlockHeader {
            state_root: spec.zero_hash,
            signature: Signature::empty_signature(),
            ..self.block_header()
        }
    }
}
