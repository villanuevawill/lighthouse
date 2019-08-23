use crate::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct PersistentCommittee<'a> {
    pub epoch: Epoch,
    pub shard: Shard,
    pub committee: &'a [usize],
}

impl<'a> PersistentCommittee<'a> {
    pub fn into_owned(self) -> OwnedPersistentCommittee {
        OwnedPersistentCommittee {
            epoch: self.epoch,
            shard: self.shard,
            committee: self.committee.to_vec(),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct OwnedPersistentCommittee {
    pub epoch: Epoch,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
