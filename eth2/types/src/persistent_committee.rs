use crate::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct PersistentCommittee {
    pub epoch: Epoch,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
