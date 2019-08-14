use serde_derive::Serialize;
use ssz_derive::{Decode, Encode};
use types::{ShardBlock, ShardState, EthSpec, Hash256};

/// Represents some block and it's associated state. Generally, this will be used for tracking the
/// head, justified head and finalized head.
#[derive(Clone, Serialize, PartialEq, Debug, Encode, Decode)]
pub struct CheckPoint<E: EthSpec> {
    pub shard_block: ShardBlock,
    pub shard_block_root: Hash256,
    pub shard_state: ShardState<E>,
    pub shard_state_root: Hash256,
}

impl<E: EthSpec> CheckPoint<E> {
    /// Create a new checkpoint.
    pub fn new(
        shard_block: ShardBlock,
        shard_block_root: Hash256,
        shard_state: ShardState<E>,
        shard_state_root: Hash256,
    ) -> Self {
        Self {
            shard_block,
            shard_block_root,
            shard_state,
            shard_state_root,
        }
    }

    /// Update all fields of the checkpoint.
    pub fn update(
        &mut self,
        shard_block: ShardBlock,
        shard_block_root: Hash256,
        shard_state: ShardState<E>,
        shard_state_root: Hash256,
    ) {
        self.shard_block = shard_block;
        self.shard_block_root = shard_block_root;
        self.shard_state = shard_state;
        self.shard_state_root = shard_state_root;
    }
}
