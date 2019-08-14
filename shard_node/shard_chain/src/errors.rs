use crate::fork_choice::Error as ForkChoiceError;
use crate::metrics::Error as MetricsError;
use state_processing::BlockProcessingError;
use state_processing::SlotProcessingError;
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
    ShardStateError(BeaconStateError),
    DBInconsistent(String),
    DBError(store::Error),
    ForkChoiceError(ForkChoiceError),
    MissingShardBlock(Hash256),
    MissingShardState(Hash256),
    SlotProcessingError(SlotProcessingError),
    MetricsError(String),
}

easy_from_to!(SlotProcessingError, ShardChainError);

impl From<MetricsError> for ShardChainError {
    fn from(e: MetricsError) -> ShardChainError {
        ShardChainError::MetricsError(format!("{:?}", e))
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockProductionError {
    UnableToGetBlockRootFromState,
    UnableToReadSlot,
    SlotProcessingError(SlotProcessingError),
    BlockProcessingError(BlockProcessingError),
    ShardStateError(ShardStateError),
}

easy_from_to!(BlockProcessingError, BlockProductionError);
easy_from_to!(ShardStateError, BlockProductionError);
easy_from_to!(SlotProcessingError, BlockProductionError);
