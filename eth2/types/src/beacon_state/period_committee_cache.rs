use super::BeaconState;
use crate::*;
use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PeriodCommitteeCache {
    pub committees: Vec<PeriodCommittee>,
}

impl PeriodCommitteeCache {
    pub fn initialize<T: EthSpec>(
        state: &BeaconState<T>,
        spec: &ChainSpec,
    ) -> Result<PeriodCommitteeCache, Error> {
        let current_epoch = state.current_epoch();
        if (current_epoch + 1) % spec.epochs_per_shard_period != 0 {
            return Err(Error::NoPeriodBoundary);
        }
        let shard_count = T::shard_count();
        let mut committees: Vec<PeriodCommittee> = Vec::with_capacity(shard_count);

        for n in 0..shard_count {
            let committee_indices = state
                .get_crosslink_committee_for_shard(n as u64, RelativeEpoch::Current)?
                .committee[..spec.target_period_committee_size]
                .to_vec();
            let period_committee = PeriodCommittee {
                shard: n as u64,
                period: current_epoch.period(spec.epochs_per_shard_period),
                committee: committee_indices,
            };
            committees.push(period_committee);
        }

        Ok(PeriodCommitteeCache { committees })
    }

    pub fn get_period_committee(&self, shard: u64) -> Result<&PeriodCommittee, Error> {
        if (self.committees.len() - 1) < shard as usize {
            return Err(Error::ShardOutOfBounds);
        }

        Ok(&self.committees[shard as usize])
    }
}
