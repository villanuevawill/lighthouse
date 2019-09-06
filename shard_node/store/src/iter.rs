use crate::Store;
use std::borrow::Cow;
use std::sync::Arc;
use types::{ShardBlock, ShardState, ShardStateError, EthSpec, Hash256, ShardSlot};

#[derive(Clone)]
pub struct StateRootsIterator<'a, T: EthSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: EthSpec, U: Store> StateRootsIterator<'a, T, U> {
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: Slot) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: start_slot + 1,
        }
    }

    pub fn owned(store: Arc<U>, beacon_state: BeaconState<T>, start_slot: Slot) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: start_slot + 1,
        }
    }
}

impl<'a, T: EthSpec, U: Store> Iterator for StateRootsIterator<'a, T, U> {
    type Item = (Hash256, Slot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == 0) || (self.slot > self.shard_state.slot) {
            return None;
        }

        self.slot -= 1;

        match self.shard_state.get_state_root(self.slot) {
            Ok(root) => Some((*root, self.slot)),
            Err(ShardStateError::SlotOutOfBounds) => {
                // Read a `BeaconState` from the store that has access to prior historical root.
                let shard_state: ShardState<T> = {
                    let new_state_root = self.shard_state.get_oldest_state_root().ok()?;

                    self.store.get(&new_state_root).ok()?
                }?;

                self.shard_state = Cow::Owned(shard_state);

                let root = self.shard_state.get_state_root(self.slot).ok()?;

                Some((*root, self.slot))
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
/// Extends `BlockRootsIterator`, returning `BeaconBlock` instances, instead of their roots.
pub struct BlockIterator<'a, T: EthSpec, U> {
    roots: BlockRootsIterator<'a, T, U>,
}

impl<'a, T: EthSpec, U: Store> BlockIterator<'a, T, U> {
    /// Create a new iterator over all blocks in the given `beacon_state` and prior states.
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            roots: BlockRootsIterator::new(store, shard_state, start_slot),
        }
    }

    /// Create a new iterator over all blocks in the given `beacon_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            roots: BlockRootsIterator::owned(store, shard_state, start_slot),
        }
    }
}

impl<'a, T: EthSpec, U: Store> Iterator for BlockIterator<'a, T, U> {
    type Item = ShardBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let (root, _slot) = self.roots.next()?;
        self.roots.store.get(&root).ok()?
    }
}

#[derive(Clone)]
pub struct BlockRootsIterator<'a, T: EthSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: EthSpec, U: Store> BlockRootsIterator<'a, T, U> {
    /// Create a new iterator over all block roots in the given `shard_state` and prior states.
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: start_slot + 1,
        }
    }

    /// Create a new iterator over all block roots in the given `beacon_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: start_slot + 1,
        }
    }
}

impl<'a, T: EthSpec, U: Store> Iterator for BlockRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == 0) || (self.slot > self.shard_state.slot) {
            return None;
        }

        self.slot -= 1;

        match self.shard_state.get_block_root(self.slot) {
            Ok(root) => Some((*root, self.slot)),
            Err(ShardStateError::SlotOutOfBounds) => {
                // Read a `ShardState` from the store that has access to prior historical root.
                let shard_state: ShardState<T> = {
                    // Load the earliest state from disk.
                    let new_state_root = self.shard_state.get_oldest_state_root().ok()?;

                    self.store.get(&new_state_root).ok()?
                }?;

                self.shard_state = Cow::Owned(shard_state);

                let root = self.shard_state.get_block_root(self.slot).ok()?;

                Some((*root, self.slot))
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct BestBlockRootsIterator<'a, T: EthSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: EthSpec, U: Store> BestBlockRootsIterator<'a, T, U> {
    pub fn new(store: Arc<U>, beacon_state: &'a ShardState<T>, start_slot: ShardSlot) -> Self {
        let mut slot = start_slot;
        if slot >= shard_state.slot {
            // Slot may be too high.
            slot = shard_state.slot;
            if shard_state.get_block_root(slot).is_err() {
                slot -= 1;
            }
        }

        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: slot + 1,
        }
    }

    /// Create a new iterator over all block roots in the given `beacon_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: ShardSlot) -> Self {
        let mut slot = start_slot;
        if slot >= shard_state.slot {
            // Slot may be too high.
            slot = shard_state.slot;
            // TODO: Use a function other than `get_block_root` as this will always return `Err()`
            // for slot = state.slot.
            if shard_state.get_block_root(slot).is_err() {
                slot -= 1;
            }
        }

        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: slot + 1,
        }
    }
}

impl<'a, T: EthSpec, U: Store> Iterator for BestBlockRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slot == 0 {
            // End of Iterator
            return None;
        }

        self.slot -= 1;

        match self.shard_state.get_block_root(self.slot) {
            Ok(root) => Some((*root, self.slot)),
            Err(ShardStateError::SlotOutOfBounds) => {
                // Read a `BeaconState` from the store that has access to prior historical root.
                let shard_state: ShardState<T> = {
                    // Load the earliest state from disk.
                    let new_state_root = self.shard_state.get_oldest_state_root().ok()?;

                    self.store.get(&new_state_root).ok()?
                }?;

                self.shard_state = Cow::Owned(shard_state);

                let root = self.shard_state.get_block_root(self.slot).ok()?;

                Some((*root, self.slot))
            }
            _ => None,
        }
    }
}