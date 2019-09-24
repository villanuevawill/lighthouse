use crate::{ShardChain, ShardChainError, ShardChainTypes};
use beacon_chain::BeaconChainTypes;
use shard_lmd_ghost::LmdGhost;
use shard_store::{Error as StoreError, Store};
use state_processing::common::get_shard_attesting_indices;
use std::sync::Arc;
use store::{Error as BeaconStoreError, Store as BeaconStore};
use types::{
    BeaconBlock, BeaconState, BeaconStateError, Epoch, EthSpec, Hash256, ShardAttestation,
    ShardBlock, ShardSlot, ShardSpec, ShardState, ShardStateError,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingBlock(Hash256),
    MissingState(Hash256),
    MissingBeaconState(Hash256),
    MissingBeaconBlock(Hash256),
    BackendError(String),
    ShardStateError(ShardStateError),
    BeaconStateError(BeaconStateError),
    StoreError(StoreError),
    BeaconStoreError(BeaconStoreError),
}

pub struct ForkChoice<T: ShardChainTypes> {
    store: Arc<T::Store>,
    backend: T::LmdGhost,
    genesis_block_root: Hash256,
}

impl<T: ShardChainTypes> ForkChoice<T> {
    pub fn new(
        store: Arc<T::Store>,
        genesis_block: &ShardBlock,
        genesis_block_root: Hash256,
    ) -> Self {
        Self {
            store: store.clone(),
            backend: T::LmdGhost::new(store, genesis_block, genesis_block_root),
            genesis_block_root,
        }
    }

    pub fn find_head<L: BeaconChainTypes>(&self, chain: &ShardChain<T, L>) -> Result<Hash256> {
        let beacon_state = chain.parent_beacon.current_state();
        let current_crosslink = beacon_state.get_current_crosslink(chain.shard)?;

        let start_block_root = current_crosslink.crosslink_data_root;
        let start_block_slot =
            ShardSlot::from(current_crosslink.end_epoch.as_u64() * chain.spec.shard_slots_per_epoch);

        // Resolve the `0x00.. 00` alias back to genesis
        let start_block_root = if start_block_root == Hash256::zero() {
            self.genesis_block_root
        } else {
            start_block_root
        };

        // A function that returns the weight for some validator index.
        let weight = |validator_index: usize| -> Option<u64> {
            beacon_state
                .validators
                .get(validator_index)
                .map(|v| v.effective_balance)
        };

        self.backend
            .find_head(start_block_slot, start_block_root, weight)
            .map_err(Into::into)
    }

    /// Process all attestations in the given `block`.
    ///
    /// Assumes the block (and therefore it's attestations) are valid. It is a logic error to
    /// provide an invalid block.
    pub fn process_block<P: EthSpec>(
        &self,
        beacon_state: &BeaconState<P>,
        block: &ShardBlock,
        block_root: Hash256,
    ) -> Result<()> {
        let attestation = &block.attestation;
        if attestation.is_empty() {
            if let Some(block) = self
                .store
                .get::<ShardBlock>(&attestation.data.shard_block_root)?
            {
                self.process_attestation(beacon_state, attestation, &block)?;
            }
        }

        self.backend.process_block(block, block_root)?;

        Ok(())
    }

    fn process_attestation<P: EthSpec>(
        &self,
        beacon_state: &BeaconState<P>,
        attestation: &ShardAttestation,
        block: &ShardBlock,
    ) -> Result<()> {
        let block_hash = attestation.data.shard_block_root;

        let validator_indices = get_shard_attesting_indices(
            block.shard,
            beacon_state,
            &attestation.data,
            &attestation.aggregation_bitfield,
        )?;

        if block_hash != Hash256::zero() {
            for validator_index in validator_indices {
                self.backend
                    .process_attestation(validator_index, block_hash, block.slot)?;
            }
        }

        Ok(())
    }

    /// Returns the latest message for a given validator, if any.
    ///
    /// Returns `(block_root, block_slot)`.
    pub fn latest_message(&self, validator_index: usize) -> Option<(Hash256, ShardSlot)> {
        self.backend.latest_message(validator_index)
    }

    /// Runs an integrity verification function on the underlying fork choice algorithm.
    ///
    /// Returns `Ok(())` if the underlying fork choice has maintained it's integrity,
    /// `Err(description)` otherwise.
    pub fn verify_integrity(&self) -> core::result::Result<(), String> {
        self.backend.verify_integrity()
    }

    /// Inform the fork choice that the given block (and corresponding root) have been finalized so
    /// it may prune it's storage.
    ///
    /// `finalized_block_root` must be the root of `finalized_block`.
    pub fn process_finalization(
        &self,
        finalized_block: &ShardBlock,
        finalized_block_root: Hash256,
    ) -> Result<()> {
        self.backend
            .update_finalized_root(finalized_block, finalized_block_root)
            .map_err(Into::into)
    }
}

impl From<ShardStateError> for Error {
    fn from(e: ShardStateError) -> Error {
        Error::ShardStateError(e)
    }
}

impl From<BeaconStateError> for Error {
    fn from(e: BeaconStateError) -> Error {
        Error::BeaconStateError(e)
    }
}

impl From<StoreError> for Error {
    fn from(e: StoreError) -> Error {
        Error::StoreError(e)
    }
}

impl From<BeaconStoreError> for Error {
    fn from(e: BeaconStoreError) -> Error {
        Error::BeaconStoreError(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::BackendError(e)
    }
}
