use crate::fork_choice::Error as ForkChoiceError;
// use crate::metrics::Error as MetricsError;
use shard_state_processing::ShardBlockProcessingError;
use shard_state_processing::ShardSlotProcessingError;
use store::Error as BeaconDBError;
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
    BeaconDBError(BeaconDBError),
    ForkChoiceError(ForkChoiceError),
    MissingShardBlock(Hash256),
    MissingShardState(Hash256),
    ShardSlotProcessingError(ShardSlotProcessingError),
    ShardBlockProcessingError(ShardBlockProcessingError),
    // MetricsError(String),
}

easy_from_to!(ShardSlotProcessingError, ShardChainError);
easy_from_to!(ShardBlockProcessingError, ShardChainError);

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
    ShardBlockProcessingError(ShardBlockProcessingError),
    BlockProcessingError(ShardBlockProcessingError),
    ShardStateError(ShardStateError),
    BeaconStateError(BeaconStateError),
}

easy_from_to!(ShardBlockProcessingError, BlockProductionError);
easy_from_to!(ShardStateError, BlockProductionError);
easy_from_to!(BeaconStateError, BlockProductionError);
easy_from_to!(BeaconStateError, ShardChainError);
easy_from_to!(ShardSlotProcessingError, BlockProductionError);

