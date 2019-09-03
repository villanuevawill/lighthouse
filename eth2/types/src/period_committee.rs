use crate::*;

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

#[derive(Default, Clone, Debug, PartialEq)]
pub struct OwnedPeriodCommittee {
    pub period: Period,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
