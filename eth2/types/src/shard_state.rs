use crate::test_utils::TestRandom;
use crate::*;
use cached_tree_hash::{Error as TreeHashCacheError, TreeHashCache};
use compare_fields_derive::CompareFields;
use hashing::hash;
use serde_derive::{Deserialize, Serialize};
use ssz::ssz_encode;
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::{CachedTreeHash, TreeHash};

#[derive(Debug, PartialEq)]
pub enum Error {
    TreeHashCacheError(TreeHashCacheError),
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
pub struct ShardState{
    pub shard: u64,
    pub slot: ShardSlot,

    #[serde(skip_serializing, skip_deserializing)]
    #[ssz(skip_serializing)]
    #[ssz(skip_deserializing)]
    #[tree_hash(skip_hashing)]
    #[test_random(default)]
    pub tree_hash_cache: TreeHashCache,
}

impl ShardState {
    pub fn genesis(
        spec: &ChainSpec,
        shard: u64,
    ) -> ShardState {
        ShardState {
            shard,
            slot: ShardSlot::from(spec.phase_1_fork_slot),
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

impl From<TreeHashCacheError> for Error {
    fn from(e: TreeHashCacheError) -> Error {
        Error::TreeHashCacheError(e)
    }
}
