#[macro_use]
mod macros;

pub mod common;

pub mod per_shard_block_processing;
pub mod per_shard_slot_processing;

pub use per_shard_block_processing::{
    errors::{Error as ShardBlockProcessingError},
    per_shard_block_processing, process_shard_block_header, 
};

pub use per_shard_slot_processing::{
    errors::{Error as ShardSlotProcessingError},
    per_shard_slot_processing, 
};
