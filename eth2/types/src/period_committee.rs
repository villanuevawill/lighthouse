use crate::*;
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    TreeHash,
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
