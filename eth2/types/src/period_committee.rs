use crate::*;
use tree_hash_derive::{CachedTreeHash, TreeHash};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, TreeHash, CachedTreeHash)]
pub struct PeriodCommittee<'a> {
    pub period: Period,
    pub shard: Shard,
    pub committee: &'a [usize],
}

impl<'a> PeriodCommittee<'a> {
    pub fn into_owned(self) -> OwnedPeriodCommittee {
        OwnedPeriodCommittee {
            period: self.period,
            shard: self.shard,
            committee: self.committee.to_vec(),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, TreeHash, CachedTreeHash)]
pub struct OwnedPeriodCommittee {
    pub period: Period,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
