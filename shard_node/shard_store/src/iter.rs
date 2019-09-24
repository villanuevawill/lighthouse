use crate::Store;
use std::borrow::Cow;
use std::sync::Arc;
use types::{Hash256, ShardBlock, ShardSlot, ShardSpec, ShardState};

/// Implemented for types that have ancestors (e.g., blocks, states) that may be iterated over.
///
/// ## Note
///
/// It is assumed that all ancestors for this object are stored in the database. If this is not the
/// case, the iterator will start returning `None` prior to genesis.
pub trait AncestorIter<U: Store, I: Iterator> {
    /// Returns an iterator over the roots of the ancestors of `self`.
    fn try_iter_ancestor_roots(&self, store: Arc<U>) -> Option<I>;
}

impl<'a, T: ShardSpec, U: Store> AncestorIter<U, BlockRootsIterator<'a, E, U>> for ShardBlock {
    /// Iterates across all available prior block roots of `self`, starting at the most recent and ending
    /// at genesis.
    fn try_iter_ancestor_roots(&self, store: Arc<U>) -> Option<BlockRootsIterator<'a, E, U>> {
        let state = store.get::<ShardState<E>>(&self.state_root).ok()??;

        Some(BlockRootsIterator::owned(store, state))
    }
}

impl<'a, T: ShardSpec, U: Store> AncestorIter<U, StateRootsIterator<'a, E, U>> for ShardState<E> {
    /// Iterates across all available prior state roots of `self`, starting at the most recent and ending
    /// at genesis.
    fn try_iter_ancestor_roots(&self, store: Arc<U>) -> Option<StateRootsIterator<'a, E, U>> {
        // The `self.clone()` here is wasteful.
        Some(StateRootsIterator::owned(store, self.clone()))
    }
}

#[derive(Clone)]
pub struct StateRootsIterator<'a, T: ShardSpec, U> {
    store: Arc<U>,
    shard_state: Cow<'a, ShardState<T>>,
    slot: ShardSlot,
}

impl<'a, T: ShardSpec, U: Store> StateRootsIterator<'a, T, U> {
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: shard_state.slot,
        }
    }

    pub fn owned(store: Arc<U>, shard_state: ShardState<T>) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: shard_state.slot,
        }
    }
}

impl<'a, T: ShardSpec, U: Store> Iterator for StateRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == T::default_spec().phase_1_fork_slot) || (self.slot > self.shard_state.slot) {
            return None;
        }

        self.slot -= 1;
        let mut next_root = self.shard_state.history_accumulator[0];

        // Efficiency gain if using log search via the accumulator instead
        while self.slot < self.shard_state.slot {
            next_root = self.shard_state.history_accumulator[0];
            let shard_state: ShardState<T> = self.store.get(&next_root).ok()??;
            self.shard_state = Cow::Owned(shard_state);
        }

        Some((next_root, self.slot))
    }
}

#[derive(Clone)]
pub struct BlockIterator<'a, T: ShardSpec, U> {
    roots: BlockRootsIterator<'a, T, U>,
}

impl<'a, T: ShardSpec, U: Store> BlockIterator<'a, T, U> {
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>) -> Self {
        Self {
            roots: BlockRootsIterator::new(store, shard_state, shard_state.slot),
        }
    }

    pub fn owned(store: Arc<U>, shard_state: ShardState<T>) -> Self {
        Self {
            roots: BlockRootsIterator::owned(store, shard_state, shard_state.slot),
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
    pub fn new(store: Arc<U>, shard_state: &'a ShardState<T>) -> Self {
        Self {
            store,
            shard_state: Cow::Borrowed(shard_state),
            slot: shard_state.slot,
        }
    }

    /// Create a new iterator over all block roots in the given `shard_state` and prior states.
    pub fn owned(store: Arc<U>, shard_state: ShardState<T>) -> Self {
        Self {
            store,
            shard_state: Cow::Owned(shard_state),
            slot: shard_state.slot
        }
    }
}

impl<'a, T: ShardSpec, U: Store> Iterator for BlockRootsIterator<'a, T, U> {
    type Item = (Hash256, ShardSlot);

    fn next(&mut self) -> Option<Self::Item> {
        if (self.slot == T::default_spec().phase_1_fork_slot) || (self.slot > self.shard_state.slot) {
            return None;
        }

        self.slot -= 1;

        // Efficiency gain if using log search via the accumulator instead
        while self.slot < self.shard_state.slot {
            let next_root = self.shard_state.history_accumulator[0];
            let shard_state: ShardState<T> = self.store.get(&next_root).ok()??;
            self.shard_state = Cow::Owned(shard_state);
        }

        let mut latest_block_header = self.shard_state.latest_block_header.clone();
        // zero out the state root to find where it was stored
        latest_block_header.state_root = Hash256::zero();
        Some((
            latest_block_header.canonical_root(),
            self.slot,
        ))
    }
}
