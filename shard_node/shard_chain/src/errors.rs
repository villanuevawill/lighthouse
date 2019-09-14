// use crate::fork_choice::Error as ForkChoiceError;
// use crate::metrics::Error as MetricsError;
use state_processing::BlockProcessingError;
use state_processing::ShardSlotProcessingError;
use types::*;

macro_rules! easy_from_to {
    ($from: ident, $to: ident) => {
        impl From<$from> for $to {
            fn from(e: $from) -> $to {
                $to::$from(e)
            }
        }
    };
}

#[derive(Debug, PartialEq)]
pub enum ShardChainError {
    InsufficientValidators,
    BadRecentBlockRoots,
    UnableToReadSlot,
    BeaconStateError(BeaconStateError),
    ShardStateError(ShardStateError),
    DBInconsistent(String),
    DBError(shard_store::Error),
    // ForkChoiceError(ForkChoiceError),
    MissingShardBlock(Hash256),
    MissingShardState(Hash256),
    ShardSlotProcessingError(ShardSlotProcessingError),
    // MetricsError(String),
}

easy_from_to!(ShardSlotProcessingError, ShardChainError);

// impl From<MetricsError> for ShardChainError {
//     fn from(e: MetricsError) -> ShardChainError {
//         ShardChainError::MetricsError(format!("{:?}", e))
//     }
// }

#[derive(Debug, PartialEq)]
pub enum BlockProductionError {
    UnableToGetBlockRootFromState,
    UnableToReadSlot,
    ShardSlotProcessingError(ShardSlotProcessingError),
    BlockProcessingError(BlockProcessingError),
    ShardStateError(ShardStateError),
    BeaconStateError(BeaconStateError),
}

easy_from_to!(BlockProcessingError, BlockProductionError);
easy_from_to!(ShardStateError, BlockProductionError);
easy_from_to!(BeaconStateError, BlockProductionError);
easy_from_to!(BeaconStateError, ShardChainError);
easy_from_to!(ShardSlotProcessingError, BlockProductionError);
