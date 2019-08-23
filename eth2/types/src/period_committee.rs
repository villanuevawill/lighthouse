use crate::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct PeriodCommittee<'a> {
    pub epoch: Epoch,
    pub shard: Shard,
    pub committee: &'a [usize],
}

impl<'a> PeriodCommittee<'a> {
    pub fn into_owned(self) -> OwnedPeriodCommittee {
        OwnedPeriodCommittee {
            epoch: self.epoch,
            shard: self.shard,
            committee: self.committee.to_vec(),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct OwnedPeriodCommittee {
    pub epoch: Epoch,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
