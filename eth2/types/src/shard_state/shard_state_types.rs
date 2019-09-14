use crate::*;
use fixed_len_vec::typenum::{U64};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait ShardSpec: 'static + Default + Sync + Send + Clone + Debug + PartialEq {
    type HistoryAccumulatorDepth: Unsigned + Clone + Sync + Send + Debug + PartialEq;

    fn default_spec() -> ChainSpec;

    fn history_accumulator_depth() -> usize {
        Self::HistoryAccumulatorDepth::to_usize()
    }
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct MainnetShardSpec;

impl ShardSpec for MainnetShardSpec {
    type HistoryAccumulatorDepth = U64;

    fn default_spec() -> ChainSpec {
        ChainSpec::mainnet()
    }
}

pub type FoundationShardState = ShardState<MainnetShardSpec>;

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct MinimalShardSpec;

impl ShardSpec for MinimalShardSpec {
    type HistoryAccumulatorDepth = U64;

    fn default_spec() -> ChainSpec {
        ChainSpec::minimal()
    }
}

pub type MinimalShardState = ShardState<MinimalShardSpec>;
