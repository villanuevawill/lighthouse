use crate::test_utils::TestRandom;
use crate::*;
use bls::Signature;

use ssz_types::typenum::U1
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::{SignedRoot, TreeHash};
use tree_hash_derive::{SignedRoot, TreeHash};

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    SignedRoot,
)]
pub struct ShardBlock {
    pub slot: ShardSlot,
    pub shard: u64,
    pub parent_root: Hash256,
    pub beacon_block_root: Hash256,
    pub state_root: Hash256,
    // add body
    pub attestation: VariableList<ShardAttestation, U1>,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

impl ShardBlock {
    pub fn empty(spec: &ChainSpec, shard: u64) -> ShardBlock {
        ShardBlock {
            shard,
            slot: ShardSlot::from(spec.phase_1_fork_slot),
            beacon_block_root: Hash256::zero(),
            parent_root: Hash256::zero(),
            state_root: Hash256::zero(),
            attestation: VariableList::empty(),
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
            beacon_block_root: self.beacon_block_root,
            parent_root: self.parent_root,
            state_root: self.state_root,
            attestation: self.attestation.clone(),
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
