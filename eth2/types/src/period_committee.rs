use crate::*;
use tree_hash_derive::{CachedTreeHash, TreeHash};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, TreeHash, Serialize, Deserialize, CachedTreeHash)]
pub struct PeriodCommittee {
    pub period: Period,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
