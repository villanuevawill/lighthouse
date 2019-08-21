use crate::*;
use tree_hash_derive::{CachedTreeHash, TreeHash};

#[derive(Default, Clone, Debug, PartialEq, TreeHash, CachedTreeHash)]
pub struct PeriodCommittee<'a> {
    pub slot: Slot,
    pub shard: Shard,
    pub committee: &'a [usize],
}

impl<'a> PeriodCommittee<'a> {
    pub fn into_owned(self) -> OwnedPeriodCommittee {
        OwnedPeriodCommittee {
            slot: self.slot,
            shard: self.shard,
            committee: self.committee.to_vec(),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, TreeHash, CachedTreeHash)]
pub struct OwnedPeriodCommittee {
    pub slot: Slot,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
