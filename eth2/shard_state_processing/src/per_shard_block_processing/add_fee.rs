use crate::*;
use types::*;

pub fn add_fee<T: ShardState, U: BeaconState>(
    state: &ShardState, 
    beacon_state: &BeaconState,
    index: u64,
) -> Result<(), Error> {
    // let epoch = self.current_epoch();
    // 
    // let earlier_committee = &self
    //     .get_period_committee(RelativePeriod::Previous, shard)?
    //     .committee;

    // let later_committee = &self
    //     .get_period_committee(RelativePeriod::Current, shard)?
    //     .committee;

    // if index in earlier_committee {
    //     state.earlier_committee_fees[earlier_committee.index(index)] += delta
    // } else {
    //     state.later_committee_fees[later_committee.index(index)] += delta
    // };

    Ok(())
}


