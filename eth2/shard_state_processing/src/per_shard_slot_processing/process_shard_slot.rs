use crate::*;
use tree_hash::TreeHash;
use types::*;
// may need to import Errors here 

pub fn process_shard_slot<T: ShardSpec>(state: &mut ShardState<T>, spec: &ChainSpec) -> () {
    let previous_state_root = Hash256::from_slice(&state.tree_hash_root()[..]);

    if state.latest_block_header.state_root == spec.zero_hash {
        state.latest_block_header.state_root = previous_state_root;
    }

    let mut depth = 0;
    while (state.slot.as_u64() % u64::pow(2, depth as u32) == 0 as u64)
        && (depth < T::history_accumulator_depth() as u64)
    {
        state.history_accumulator[depth as usize] = previous_state_root;
        depth += 1;
    }
}
