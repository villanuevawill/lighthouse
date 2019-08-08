use crate::Store;
use std::borrow::Cow;
use std::sync::Arc;
use types::{ShardBlock, ShardState, ShardStateError, EthSpec, Hash256, Slot};

#[derive(Clone)]
pub struct StateRootsIterator<'a, T: EthSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: Slot,
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
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: Slot) -> Self {
        Self {
            roots: BlockRootsIterator::new(store, shard_state, start_slot),
        }
    }

    /// Create a new iterator over all blocks in the given `beacon_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: Slot) -> Self {
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

/// Iterates backwards through block roots. If any specified slot is unable to be retrieved, the
/// iterator returns `None` indefinitely.
///
/// Uses the `latest_block_roots` field of `BeaconState` to as the source of block roots and will
/// perform a lookup on the `Store` for a prior `BeaconState` if `latest_block_roots` has been
/// exhausted.
///
/// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
///
/// ## Notes
///
/// See [`BestBlockRootsIterator`](struct.BestBlockRootsIterator.html), which has different
/// `start_slot` logic.
#[derive(Clone)]
pub struct BlockRootsIterator<'a, T: EthSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: Slot,
}

impl<'a, T: EthSpec, U: Store> BlockRootsIterator<'a, T, U> {
    /// Create a new iterator over all block roots in the given `shard_state` and prior states.
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>, start_slot: Slot) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: start_slot + 1,
        }
    }

    /// Create a new iterator over all block roots in the given `beacon_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: Slot) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: start_slot + 1,
        }
    }
}

impl<'a, T: EthSpec, U: Store> Iterator for BlockRootsIterator<'a, T, U> {
    type Item = (Hash256, Slot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == 0) || (self.slot > self.shard_state.slot) {
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

/// Iterates backwards through block roots with `start_slot` highest possible value
/// `<= beacon_state.slot`.
///
/// The distinction between `BestBlockRootsIterator` and `BlockRootsIterator` is:
///
/// - `BestBlockRootsIterator` uses best-effort slot. When `start_slot` is greater than the latest available block root
/// on `beacon_state`, returns `Some(root, slot)` where `slot` is the latest available block
/// root.
/// - `BlockRootsIterator` is strict about `start_slot`. When `start_slot` is greater than the latest available block root
/// on `beacon_state`, returns  `None`.
///
/// This is distinct from `BestBlockRootsIterator`.
///
/// Uses the `latest_block_roots` field of `BeaconState` to as the source of block roots and will
/// perform a lookup on the `Store` for a prior `BeaconState` if `latest_block_roots` has been
/// exhausted.
///
/// Returns `None` for roots prior to genesis or when there is an error reading from `Store`.
#[derive(Clone)]
pub struct BestBlockRootsIterator<'a, T: EthSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: Slot,
}

impl<'a, T: EthSpec, U: Store> BestBlockRootsIterator<'a, T, U> {
    /// Create a new iterator over all block roots in the given `beacon_state` and prior states.
    pub fn new(store: Arc<U>, beacon_state: &'a ShardState<T>, start_slot: Slot) -> Self {
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
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>, start_slot: Slot) -> Self {
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
    type Item = (Hash256, Slot);

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