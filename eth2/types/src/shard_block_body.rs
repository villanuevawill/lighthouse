use crate::test_utils::{graffiti_from_hex_str, TestRandom};
use crate::*;

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::{CachedTreeHash, TreeHash};

/// The body of a `ShardChain` block, containing operations.
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
)]
pub struct ShardBlockBody {
    #[serde(deserialize_with = "graffiti_from_hex_str")]
    pub graffiti: [u8; 32],
    pub attestation: ShardAttestation,
}
