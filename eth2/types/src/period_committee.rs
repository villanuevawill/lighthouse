use crate::*;
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::{CachedTreeHash, TreeHash};

#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    TreeHash,
    CachedTreeHash,
    Serialize,
    Deserialize,
    Decode,
    Encode,
)]
pub struct PeriodCommittee {
    pub period: Period,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
