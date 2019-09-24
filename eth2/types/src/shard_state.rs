use crate::test_utils::TestRandom;
use crate::*;
use compare_fields_derive::CompareFields;
use serde_derive::{Deserialize, Serialize};
use ssz_types::FixedVector;
use ssz_derive::{Decode, Encode};
use std::marker::PhantomData;
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

pub use shard_state_types::*;

mod shard_state_types;

#[derive(Debug, PartialEq)]
pub enum Error {
    SszTypesError(ssz_types::Error),
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    TestRandom,
    Encode,
    Decode,
    TreeHash,
    CachedTreeHash,
    CompareFields,
)]
pub struct ShardState<T>
where
    T: ShardSpec,
{
    pub shard: u64,
    pub slot: ShardSlot,
    pub history_accumulator: FixedVector<Hash256, T::HistoryAccumulatorDepth>,
    pub latest_block_header: ShardBlockHeader,

    #[serde(skip_serializing, skip_deserializing)]
    #[ssz(skip_serializing)]
    #[ssz(skip_deserializing)]
    #[tree_hash(skip_hashing)]
    #[test_random(default)]
    pub tree_hash_cache: TreeHashCache,
}

impl<T: ShardSpec> ShardState<T> {
    pub fn genesis(spec: &ChainSpec, shard: u64) -> ShardState<T> {
        ShardState {
            shard,
            slot: ShardSlot::from(spec.phase_1_fork_slot),
            history_accumulator: FixedLenVec::from(vec![
                spec.zero_hash;
                T::HistoryAccumulatorDepth::to_usize()
            ]),
            latest_block_header: ShardBlockHeader::empty(spec, shard),
            tree_hash_cache: TreeHashCache::default(),
        }
    }

    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.tree_hash_root()[..])
    }

    pub fn build_cache(&mut self, spec: &ChainSpec) -> Result<(), Error> {
        self.update_tree_hash_cache()?;
        Ok(())
    }

    pub fn drop_cache(&mut self) {
        self.drop_tree_hash_cache();
    }

    pub fn update_tree_hash_cache(&mut self) -> Result<Hash256, Error> {
        if self.tree_hash_cache.is_empty() {
            self.tree_hash_cache = TreeHashCache::new(self)?;
        } else {
            let mut cache = std::mem::replace(&mut self.tree_hash_cache, TreeHashCache::default());
            cache.update(self)?;
            self.tree_hash_cache = cache
        }

        self.cached_tree_hash_root()
    }

    pub fn cached_tree_hash_root(&self) -> Result<Hash256, Error> {
        self.tree_hash_cache
            .tree_hash_root()
            .and_then(|b| Ok(Hash256::from_slice(b)))
            .map_err(Into::into)
    }

    pub fn drop_tree_hash_cache(&mut self) {
        self.tree_hash_cache = TreeHashCache::default()
    }
}

impl From<ssz_types::Error> for Error {
    fn from(e: ssz_types::Error) -> Error {
        Error::SszTypesError(e)
    }
}
