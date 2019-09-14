use crate::Store;
use std::borrow::Cow;
use std::sync::Arc;
use types::{ShardBlock, ShardState, ShardStateError, ShardSpec, Hash256, ShardSlot};

#[derive(Clone)]
pub struct StateRootsIterator<'a, T: ShardSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: ShardSpec, U: Store> StateRootsIterator<'a, T, U> {
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: start_slot + 1,
        }
    }

    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: start_slot + 1,
        }
    }
}

impl<'a, T: ShardSpec, U: Store> Iterator for StateRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == 0) || (self.slot > self.shard_state.slot) {
            return None;
        }

        self.slot -= 1;

        // Efficiency gain if using log search via the accumulator instead
        while self.slot < self.shard_state.slot {
            let next_root = self.shard_state.history_accumulator[0];
            let shard_state: ShardState<T> = self.store.get(&next_root).ok()??;

            if self.slot > shard_state.slot {
                return Some((Hash256::zero(), self.slot));
            }

            self.shard_state = Cow::Owned(shard_state);
        }

        Some((self.shard_state.latest_block_header.state_root, self.slot))
    }
}

#[derive(Clone)]
pub struct BlockIterator<'a, T: ShardSpec, U> {
    roots: BlockRootsIterator<'a, T, U>,
}

impl<'a, T: ShardSpec, U: Store> BlockIterator<'a, T, U> {
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            roots: BlockRootsIterator::new(store, shard_state, start_slot),
        }
    }

    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            roots: BlockRootsIterator::owned(store, shard_state, start_slot),
        }
    }
}

impl<'a, T: ShardSpec, U: Store> Iterator for BlockIterator<'a, T, U> {
    type Item = ShardBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let (root, _slot) = self.roots.next()?;
        self.roots.store.get(&root).ok()?
    }
}

#[derive(Clone)]
pub struct BlockRootsIterator<'a, T: ShardSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: ShardSpec, U: Store> BlockRootsIterator<'a, T, U> {
    /// Create a new iterator over all block roots in the given `shard_state` and prior states.
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: start_slot + 1,
        }
    }

    /// Create a new iterator over all block roots in the given `shard_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: start_slot + 1,
        }
    }
}

impl<'a, T: ShardSpec, U: Store> Iterator for BlockRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == 0) || (self.slot > self.shard_state.slot) {
            return None;
        }

        self.slot -= 1;

        // Efficiency gain if using log search via the accumulator instead
        while self.slot < self.shard_state.slot {
            let next_root = self.shard_state.history_accumulator[0];
            let shard_state: ShardState<T> = self.store.get(&next_root).ok()??;

            if self.slot > shard_state.slot {
                return Some((Hash256::zero(), self.slot));
            }

            self.shard_state = Cow::Owned(shard_state);
        }

        Some((self.shard_state.latest_block_header.canonical_root(), self.slot))
    }
}

#[derive(Clone)]
pub struct BestBlockRootsIterator<'a, T: ShardSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: ShardSpec, U: Store> BestBlockRootsIterator<'a, T, U> {
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        let mut slot = start_slot;
        if slot >= shard_state.slot {
            slot = shard_state.slot;
        }

        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: slot + 1,
        }
    }

    /// Create a new iterator over all block roots in the given `shard_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        let mut slot = start_slot;
        if slot >= shard_state.slot {
            // Slot may be too high.
            slot = shard_state.slot;
        }

        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: slot + 1,
        }
    }
}

impl<'a, T: ShardSpec, U: Store> Iterator for BestBlockRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slot == 0 {
            // End of Iterator
            return None;
        }

        self.slot -= 1;

        // Efficiency gain if using log search via the accumulator instead
        while self.slot < self.shard_state.slot {
            let next_root = self.shard_state.history_accumulator[0];
            let shard_state: ShardState<T> = self.store.get(&next_root).ok()??;

            if self.slot > shard_state.slot {
                return Some((Hash256::zero(), self.slot));
            }

            self.shard_state = Cow::Owned(shard_state);
        }

        Some((self.shard_state.latest_block_header.canonical_root(), self.slot))
    }
}