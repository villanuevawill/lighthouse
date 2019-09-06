use crate::*;
use fixed_len_vec::typenum::{Unsigned, U0, U1024, U64, U8, U8192, U256};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait EthSpec: 'static + Default + Sync + Send + Clone + Debug + PartialEq {
    type ShardCount: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    type SlotsPerHistoricalRoot: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    type LatestRandaoMixesLength: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    type PeriodCommitteeRootsLength: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    type LatestActiveIndexRootsLength: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    type LatestSlashedExitLength: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    /// Note: `SlotsPerEpoch` is not necessarily required to be a compile-time constant. We include
    /// it here just for the convenience of not passing `slots_per_epoch` around all the time.
    type SlotsPerEpoch: Unsigned + Clone + Sync + Send + Debug + PartialEq;
    type GenesisEpoch: Unsigned + Clone + Sync + Send + Debug + PartialEq;

    fn default_spec() -> ChainSpec;

    fn genesis_epoch() -> Epoch {
        Epoch::new(Self::GenesisEpoch::to_u64())
    }

    /// Return the number of committees in one epoch.
    ///
    /// Spec v0.6.3
    fn get_epoch_committee_count(
        active_validator_count: usize,
        target_committee_size: usize,
    ) -> usize {
        let shard_count = Self::shard_count();
        let slots_per_epoch = Self::slots_per_epoch() as usize;

        std::cmp::max(
            1,
            std::cmp::min(
                shard_count / slots_per_epoch,
                active_validator_count / slots_per_epoch / target_committee_size,
            ),
        ) * slots_per_epoch
    }

    /// Return the number of shards to increment `state.latest_start_shard` by in a given epoch.
    ///
    /// Spec v0.6.3
    fn get_shard_delta(active_validator_count: usize, target_committee_size: usize) -> u64 {
        std::cmp::min(
            Self::get_epoch_committee_count(active_validator_count, target_committee_size) as u64,
            Self::ShardCount::to_u64() - Self::ShardCount::to_u64() / Self::slots_per_epoch(),
        )
    }

    /// Returns the minimum number of validators required for this spec.
    ///
    /// This is the _absolute_ minimum, the number required to make the chain operate in the most
    /// basic sense. This count is not required to provide any security guarantees regarding
    /// decentralization, entropy, etc.
    fn minimum_validator_count() -> usize {
        Self::SlotsPerEpoch::to_usize()
    }

    /// Returns the `SLOTS_PER_EPOCH` constant for this specification.
    ///
    /// Spec v0.6.3
    fn slots_per_epoch() -> u64 {
        Self::SlotsPerEpoch::to_u64()
    }

    /// Returns the `SHARD_COUNT` constant for this specification.
    ///
    /// Spec v0.6.3
    fn shard_count() -> usize {
        Self::ShardCount::to_usize()
    }

    /// Returns the `SLOTS_PER_HISTORICAL_ROOT` constant for this specification.
    ///
    /// Spec v0.6.3
    fn slots_per_historical_root() -> usize {
        Self::SlotsPerHistoricalRoot::to_usize()
    }

    /// Returns the `LATEST_RANDAO_MIXES_LENGTH` constant for this specification.
    ///
    /// Spec v0.6.3
    fn latest_randao_mixes_length() -> usize {
        Self::LatestRandaoMixesLength::to_usize()
    }

    /// Returns the `LATEST_ACTIVE_INDEX_ROOTS` constant for this specification.
    ///
    /// Spec v0.6.3
    fn latest_active_index_roots() -> usize {
        Self::LatestActiveIndexRootsLength::to_usize()
    }

    /// Returns the `LATEST_SLASHED_EXIT_LENGTH` constant for this specification.
    ///
    /// Spec v0.6.3
    fn latest_slashed_exit_length() -> usize {
        Self::LatestSlashedExitLength::to_usize()
    }
}

/// Ethereum Foundation specifications.
///
/// Spec v0.6.3
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct MainnetEthSpec;

impl EthSpec for MainnetEthSpec {
    type ShardCount = U1024;
    type SlotsPerHistoricalRoot = U8192;
    type LatestRandaoMixesLength = U8192;
    type PeriodCommitteeRootsLength = U256;
    type LatestActiveIndexRootsLength = U8192;
    type LatestSlashedExitLength = U8192;
    type SlotsPerEpoch = U64;
    type GenesisEpoch = U0;

    fn default_spec() -> ChainSpec {
        ChainSpec::mainnet()
    }
}

pub type FoundationBeaconState = BeaconState<MainnetEthSpec>;

/// Ethereum Foundation minimal spec, as defined here:
///
/// https://github.com/ethereum/eth2.0-specs/blob/v0.6.3/configs/constant_presets/minimal.yaml
///
/// Spec v0.6.3
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct MinimalEthSpec;

impl EthSpec for MinimalEthSpec {
    type ShardCount = U8;
    type SlotsPerHistoricalRoot = U64;
    type PeriodCommitteeRootsLength = U64;
    type LatestRandaoMixesLength = U1024;
    type LatestActiveIndexRootsLength = U1024;
    type LatestSlashedExitLength = U64;
    type SlotsPerEpoch = U8;
    type GenesisEpoch = U0;

    fn default_spec() -> ChainSpec {
        ChainSpec::minimal()
    }
}

pub type MinimalBeaconState = BeaconState<MinimalEthSpec>;
