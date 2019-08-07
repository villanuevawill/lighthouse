use crate::test_utils::TestRandom;
use crate::*;
use bls::Signature;

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::{SignedRoot, TreeHash};
use tree_hash_derive::{CachedTreeHash, SignedRoot, TreeHash};

/// A block of the `BeaconChain`.
///
/// Spec v0.6.3
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
    pub slot: Slot,
    pub previous_block_root: Hash256,
    pub state_root: Hash256,
    pub body: ShardBlockBody,
    #[signed_root(skip_hashing)]
    pub signature: Signature,
}

impl ShardBlock {
    /// Returns an empty block to be used during genesis.
    ///
    /// Spec v0.6.3
    pub fn empty(spec: &ChainSpec) -> BeaconBlock {
        ShardBlock {
            slot: spec.genesis_slot,
            previous_block_root: spec.zero_hash,
            state_root: spec.zero_hash,
            body: ShardBlockBody {
                graffiti: [0; 32],
                attestations: vec![],
            },
            signature: Signature::empty_signature(),
        }
    }

    /// Returns the `signed_root` of the block.
    ///
    /// Spec v0.6.3
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.signed_root()[..])
    }

    /// Returns a full `ShardBlockHeader` of this block.
    ///
    /// Note: This method is used instead of an `Into` impl to avoid a `Clone` of an entire block
    /// when you want to have the block _and_ the header.
    ///
    /// Note: performs a full tree-hash of `self.body`.
    ///
    /// Spec v0.6.3
    pub fn block_header(&self) -> ShardBlockHeader {
        ShardBlockHeader {
            slot: self.slot,
            previous_block_root: self.previous_block_root,
            state_root: self.state_root,
            block_body_root: Hash256::from_slice(&self.body.tree_hash_root()[..]),
            signature: self.signature.clone(),
        }
    }

    /// Returns a "temporary" header, where the `state_root` is `spec.zero_hash`.
    ///
    /// Spec v0.6.3
    pub fn temporary_block_header(&self, spec: &ChainSpec) -> ShardBlockHeader {
        ShardBlockHeader {
            state_root: spec.zero_hash,
            signature: Signature::empty_signature(),
            ..self.block_header()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(BeaconBlock);
    cached_tree_hash_tests!(BeaconBlock);
}
