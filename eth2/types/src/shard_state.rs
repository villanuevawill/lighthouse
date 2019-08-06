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
    ValidatorIsWithdrawable,
    UnableToShuffle,
    TooManyValidators,
    InsufficientValidators,
    InsufficientRandaoMixes,
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
    pub slot: Slot,
    pub genesis_time: u64,
    pub fork: Fork,

    // Attestations - split by epoch necessary or just from latest finalized crosslink?
    // Going to need a way to request the attestation properly by epoch... play with this later
    pub running_attestations: Vec<PendingAttestation>,

    // Latest beacon roots needed
    // Latest crosslinks not needed since it will only duplicate data accessed via beacon roots
    pub latest_beacon_roots: FixedLenVec<Hash256, T::SlotsPerHistoricalRoot>,

    // Recent state
    pub latest_block_roots: FixedLenVec<Hash256, T::SlotsPerHistoricalRoot>,
    #[compare_fields(as_slice)]
    pub latest_state_roots: FixedLenVec<Hash256, T::SlotsPerHistoricalRoot>,
    pub latest_block_header: ShardBlockHeader,
    pub historical_roots: Vec<Hash256>,
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
            // will this genesis time not matter - can we just pull from the spec?
            genesis_time,
            fork: Fork::genesis(T::genesis_epoch()),

            // Attestations
            previous_epoch_attestations: vec![],
            current_epoch_attestations: vec![],

            // Recent state
            current_crosslinks: vec![initial_crosslink.clone(); T::ShardCount::to_usize()].into(),
            previous_crosslinks: vec![initial_crosslink; T::ShardCount::to_usize()].into(),
            latest_block_roots: vec![spec.zero_hash; T::SlotsPerHistoricalRoot::to_usize()].into(),
            latest_state_roots: vec![spec.zero_hash; T::SlotsPerHistoricalRoot::to_usize()].into(),
            latest_active_index_roots: vec![
                spec.zero_hash;
                T::LatestActiveIndexRootsLength::to_usize()
            ]
            .into(),
            latest_slashed_balances: vec![0; T::LatestSlashedExitLength::to_usize()].into(),
            latest_block_header: BeaconBlock::empty(spec).temporary_block_header(spec),
            historical_roots: vec![],

            /*
             * PoW receipt root
             */
            latest_eth1_data,
            eth1_data_votes: vec![],
            deposit_index: 0,

            /*
             * Caching (not in spec)
             */
            committee_caches: [
                CommitteeCache::default(),
                CommitteeCache::default(),
                CommitteeCache::default(),
            ],
            pubkey_cache: PubkeyCache::default(),
            tree_hash_cache: TreeHashCache::default(),
            exit_cache: ExitCache::default(),
        }
    }

    /// Returns the `tree_hash_root` of the state.
    ///
    /// Spec v0.6.3
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.tree_hash_root()[..])
    }

    pub fn historical_batch(&self) -> HistoricalBatch<T> {
        HistoricalBatch {
            block_roots: self.latest_block_roots.clone(),
            state_roots: self.latest_state_roots.clone(),
        }
    }

    /// Get the slot of an attestation.
    ///
    /// Note: Utilizes the cache and will fail if the appropriate cache is not initialized.
    ///
    /// Spec v0.6.3
    pub fn get_attestation_slot(&self, attestation_data: &AttestationData) -> Result<Slot, Error> {
        let target_relative_epoch =
            RelativeEpoch::from_epoch(self.current_epoch(), attestation_data.target_epoch)?;

        let cc =
            self.get_crosslink_committee_for_shard(attestation_data.shard, target_relative_epoch)?;

        Ok(cc.slot)
    }

    /// Safely obtains the index for latest block roots, given some `slot`.
    ///
    /// Spec v0.6.3
    fn get_latest_block_roots_index(&self, slot: Slot) -> Result<usize, Error> {
        if (slot < self.slot) && (self.slot <= slot + self.latest_block_roots.len() as u64) {
            Ok(slot.as_usize() % self.latest_block_roots.len())
        } else {
            Err(BeaconStateError::SlotOutOfBounds)
        }
    }

    /// Return the block root at a recent `slot`.
    ///
    /// Spec v0.6.3
    pub fn get_block_root(&self, slot: Slot) -> Result<&Hash256, BeaconStateError> {
        let i = self.get_latest_block_roots_index(slot)?;
        Ok(&self.latest_block_roots[i])
    }

    /// Sets the block root for some given slot.
    ///
    /// Spec v0.6.3
    pub fn set_block_root(
        &mut self,
        slot: Slot,
        block_root: Hash256,
    ) -> Result<(), BeaconStateError> {
        let i = self.get_latest_block_roots_index(slot)?;
        self.latest_block_roots[i] = block_root;
        Ok(())
    }

    /// Safely obtains the index for latest state roots, given some `slot`.
    ///
    /// Spec v0.6.3
    fn get_latest_state_roots_index(&self, slot: Slot) -> Result<usize, Error> {
        if (slot < self.slot) && (self.slot <= slot + Slot::from(self.latest_state_roots.len())) {
            Ok(slot.as_usize() % self.latest_state_roots.len())
        } else {
            Err(BeaconStateError::SlotOutOfBounds)
        }
    }

    /// Gets the state root for some slot.
    ///
    /// Spec v0.6.3
    pub fn get_state_root(&self, slot: Slot) -> Result<&Hash256, Error> {
        let i = self.get_latest_state_roots_index(slot)?;
        Ok(&self.latest_state_roots[i])
    }

    /// Gets the oldest (earliest slot) state root.
    ///
    /// Spec v0.6.3
    pub fn get_oldest_state_root(&self) -> Result<&Hash256, Error> {
        let i = self
            .get_latest_state_roots_index(self.slot - Slot::from(self.latest_state_roots.len()))?;
        Ok(&self.latest_state_roots[i])
    }

    /// Sets the latest state root for slot.
    ///
    /// Spec v0.6.3
    pub fn set_state_root(&mut self, slot: Slot, state_root: Hash256) -> Result<(), Error> {
        let i = self.get_latest_state_roots_index(slot)?;
        self.latest_state_roots[i] = state_root;
        Ok(())
    }

    /// Do we need this since the tree hash really is just the balance set of everyone?
    /// Build all the caches, if they need to be built.
    pub fn build_all_caches(&mut self, spec: &ChainSpec) -> Result<(), Error> {
        self.update_tree_hash_cache()?;
        Ok(())
    }

    /// Drop all caches on the state.
    pub fn drop_all_caches(&mut self) {
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
