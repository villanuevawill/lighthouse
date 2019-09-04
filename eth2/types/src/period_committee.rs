use crate::*;
use tree_hash_derive::{CachedTreeHash, TreeHash};
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

#[derive(Default, Clone, Debug, PartialEq, TreeHash, CachedTreeHash, Serialize, Deserialize, Decode, Encode)]
pub struct PeriodCommittee {
    pub period: Period,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
