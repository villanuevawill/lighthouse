use super::BeaconState;
use crate::*;
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

/// Computes and stores the shuffling for an epoch. Provides various getters to allow callers to
/// read the committees for the given epoch.
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PeriodCommitteeCache {
    pub committee: Vec<usize>,
}

impl PeriodCommitteeCache {
    pub fn initialize<T: EthSpec>(
        state: &BeaconState<T>,
        spec: &ChainSpec,
        shard: u64,
    ) -> Result<PeriodCommitteeCache, Error> {
        let current_epoch = state.current_epoch();
        if current_epoch % spec.epochs_per_shard_period != 0 {
            return Err(Error::NoPeriodBoundary);
        }

        let committee = state.get_crosslink_committee_for_shard(shard, RelativeEpoch::Current)?.committee[..spec.target_period_committee_size].to_vec();
        Ok(PeriodCommitteeCache{committee})
    }
}
