use crate::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ShardCommittee {
    pub epoch: Epoch,
    pub shard: Shard,
    pub committee: Vec<usize>,
}
