use self::committee_cache::get_active_validator_indices;
use crate::test_utils::TestRandom;
use crate::*;
use cached_tree_hash::{Error as TreeHashCacheError, TreeHashCache};
use compare_fields_derive::CompareFields;
use fixed_len_vec::{typenum::Unsigned, FixedLenVec};
use hashing::hash;
use int_to_bytes::{int_to_bytes32, int_to_bytes8};
use pubkey_cache::PubkeyCache;
use serde_derive::{Deserialize, Serialize};
use ssz::ssz_encode;
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::{CachedTreeHash, TreeHash};

pub use self::committee_cache::CommitteeCache;
pub use beacon_state_types::*;

mod beacon_state_types;
mod committee_cache;
mod pubkey_cache;
mod tests;

pub const CACHED_EPOCHS: usize = 3;
const MAX_RANDOM_BYTE: u64 = (1 << 8) - 1;

#[derive(Debug, PartialEq)]
pub enum Error {
    EpochOutOfBounds,
    SlotOutOfBounds,
    UnknownValidator,
    UnableToDetermineProducer,
    InvalidBitfield,
    InsufficientBlockRoots,
    InsufficientIndexRoots,
    InsufficientAttestations,
    InsufficientStateRoots,
    NoCommitteeForShard,
    NoCommitteeForSlot,
    TreeHashCacheError(TreeHashCacheError),
}

/// The state of the `BeaconChain` at some slot.
///
/// Spec v0.6.3
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
    T: EthSpec,
{
    // Misc
    pub slot: ShardSlot,

    // Attestations - Update to bitfield but simply one attestation included
    pub attestation: ShardPendingAttestation,

    // Latest beacon root needed
    pub beacon_root: Hash256,

    // Parent root
    pub parent_root: Hash256,

    // Caching (not in the spec)
    #[serde(default)]
    #[ssz(skip_serializing)]
    #[ssz(skip_deserializing)]
    #[tree_hash(skip_hashing)]
    #[test_random(default)]
    pub committees: [PeriodCommittee; 3],
}

impl<T: EthSpec> ShardState<T> {
    /// Produce the first state of the Shard Chain.
    ///
    /// This does not fully build a genesis shard state, it omits processing of initial validator
    /// deposits. To obtain a full genesis shard state, use the `ShardStateBuilder`.
    ///
    /// Spec v0.6.3
    pub fn genesis(
        genesis_time: u64,
        spec: &ChainSpec,
    ) -> ShardState<T> {
        ShardState {
            // Misc
            slot: spec.genesis_slot,
            fork: Fork::genesis(T::genesis_epoch()),

            // Attestation
            attestation: ShardPendingAttestation::default(),

            // Roots
            beacon_root: Hash256::default(),
            parent_root: Hash256::default(),

            /*
             * Cacheing (not in spec)
             */
            tree_hash_cache: TreeHashCache::default(),
            committees: [PeriodCommittee::default(); 3],
        }
    }

    /// Returns the `tree_hash_root` of the state.
    ///
    /// Spec v0.6.3
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.tree_hash_root()[..])
    }

    pub fn get_earlier_committee(&self) -> PeriodCommittee {
        self.committees[0]
    }

    pub fn get_later_committee(&self) -> PeriodCommittee {
        self.committees[1]
    }

    pub fn get_next_committee(&self) -> PeriodCommittee {
        self.committees[2]
    }

    pub fn get_persistent_committee(&self) -> PersistentCommittee {
        let earlier_committee = self.get_earlier_committee().to_owned().committee;
        let later_committee = self.get_later_committee().to_owned().committee;

        let persistent_committee_indexes = {
            // loop through properly
        }
        // finish logic here - fairly simple
    }

    /// Do we need this since the tree hash really is just the balance set of everyone?
    /// Build all the caches, if they need to be built.
    pub fn build_cache(&mut self, spec: &ChainSpec) -> Result<(), Error> {
        self.update_tree_hash_cache()?;
        Ok(())
    }

    /// Drop all caches on the state.
    pub fn drop_cache(&mut self) {
        self.drop_tree_hash_cache();
    }

    /// Update the tree hash cache, building it for the first time if it is empty.
    ///
    /// Returns the `tree_hash_root` resulting from the update. This root can be considered the
    /// canonical root of `self`.
    pub fn update_tree_hash_cache(&mut self) -> Result<Hash256, Error> {
        if self.tree_hash_cache.is_empty() {
            self.tree_hash_cache = TreeHashCache::new(self)?;
        } else {
            // Move the cache outside of `self` to satisfy the borrow checker.
            let mut cache = std::mem::replace(&mut self.tree_hash_cache, TreeHashCache::default());

            cache.update(self)?;

            // Move the updated cache back into `self`.
            self.tree_hash_cache = cache
        }

        self.cached_tree_hash_root()
    }

    /// Returns the tree hash root determined by the last execution of `self.update_tree_hash_cache(..)`.
    ///
    /// Note: does _not_ update the cache and may return an outdated root.
    ///
    /// Returns an error if the cache is not initialized or if an error is encountered during the
    /// cache update.
    pub fn cached_tree_hash_root(&self) -> Result<Hash256, Error> {
        self.tree_hash_cache
            .tree_hash_root()
            .and_then(|b| Ok(Hash256::from_slice(b)))
            .map_err(Into::into)
    }

    /// Completely drops the tree hash cache, replacing it with a new, empty cache.
    pub fn drop_tree_hash_cache(&mut self) {
        self.tree_hash_cache = TreeHashCache::default()
    }
}

impl From<RelativeEpochError> for Error {
    fn from(e: RelativeEpochError) -> Error {
        Error::RelativeEpochError(e)
    }
}

impl From<TreeHashCacheError> for Error {
    fn from(e: TreeHashCacheError) -> Error {
        Error::TreeHashCacheError(e)
    }
}
