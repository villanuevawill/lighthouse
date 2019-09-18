#[macro_use]
mod macros;

pub mod common;
pub mod get_genesis_state;
pub mod per_block_processing;
pub mod per_epoch_processing;
pub mod per_shard_block_processing;
pub mod per_shard_slot_processing;
pub mod per_slot_processing;

pub use get_genesis_state::get_genesis_beacon_state;
pub use per_block_processing::{
    errors::{BlockInvalid, BlockProcessingError},
    per_block_processing, per_block_processing_without_verifying_block_signature,
};
pub use per_epoch_processing::{errors::EpochProcessingError, per_epoch_processing};
pub use per_shard_block_processing::{
    per_shard_block_processing, Error as ShardBlockProcessingError,
};
pub use per_shard_slot_processing::{per_shard_slot_processing, Error as ShardSlotProcessingError};
pub use per_slot_processing::{per_slot_processing, Error as SlotProcessingError};
