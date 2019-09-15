use crate::{ShardChain, ShardChainTypes, ShardChainError};
use beacon_chain::BeaconChainTypes;
use shard_lmd_ghost::LmdGhost;
use state_processing::common::get_shard_attesting_indices_unsorted;
use std::sync::Arc;
use store::{Error as BeaconStoreError, Store as BeaconStore};
use shard_store::{Error as StoreError, Store};
use types::{ShardAttestation, BeaconBlock, BeaconState, BeaconStateError, ShardBlock, ShardSlot, ShardState, ShardStateError, Epoch, EthSpec, ShardSpec, Hash256};

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
    backend: T::LmdGhost,
    genesis_block_root: Hash256,
}

impl<T: ShardChainTypes>ForkChoice<T> {
    pub fn new(
        store: Arc<T::Store>,
        genesis_block: &ShardBlock,
        genesis_block_root: Hash256,
    ) -> Self {
        Self {
            backend: T::LmdGhost::new(store, genesis_block, genesis_block_root),
            genesis_block_root,
        }
    }

    pub fn find_head<L: BeaconChainTypes>(&self, chain: &ShardChain<T, L>) -> Result<Hash256> {
        let current_state = chain.current_state();
        let beacon_root = current_state.latest_block_header.beacon_block_root;
        let beacon_block: BeaconBlock = chain
            .parent_beacon
            .store
            .get(&beacon_root)?
            .ok_or_else(|| Error::MissingBeaconBlock(beacon_root))?;

        let beacon_state: BeaconState<L::EthSpec> = chain
            .parent_beacon
            .store
            .get(&beacon_block.state_root)?
            .ok_or_else(|| Error::MissingBeaconState(beacon_block.state_root))?;

        let current_crosslink = beacon_state.get_current_crosslink(chain.shard)?;
        // Spec needs an update for crosslinks to hold the end shard_block_root
        // For now, we will just assume the latest block hash is included and add the
        // extra field to the beacon chain
        let start_block_root = current_crosslink.crosslink_data_root;
        // should be updated to end epoch :) with the new spec todo
        let start_block_slot = ShardSlot::from(current_crosslink.epoch.as_u64() * chain.spec.shard_slots_per_epoch);

        // A function that returns the weight for some validator index.
        let weight = |validator_index: usize| -> Option<u64> {
            beacon_state
                .validator_registry
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
        self.process_attestation_from_block(beacon_state, &block.attestation, block)?;
        self.backend.process_block(block, block_root)?;

        Ok(())
    }

    fn process_attestation_from_block<P: EthSpec>(
        &self,
        beacon_state: &BeaconState<P>,
        attestation: &ShardAttestation,
        block: &ShardBlock
    ) -> Result<()> {
        // Note: `get_attesting_indices_unsorted` requires that the beacon state caches be built.
        let validator_indices = get_shard_attesting_indices_unsorted(
            block.shard,
            beacon_state,
            &attestation.data,
            &attestation.aggregation_bitfield,
        )?;

        let block_hash = attestation.data.shard_block_root;

        if block_hash != Hash256::zero() {
            for validator_index in validator_indices {
                self.backend
                    .process_attestation(validator_index, block_hash, block.slot)?;
            }
        }

        Ok(())
    }

    // /// Inform the fork choice that the given block (and corresponding root) have been finalized so
    // /// it may prune it's storage.
    // ///
    // /// `finalized_block_root` must be the root of `finalized_block`.
    // pub fn process_finalization(
    //     &self,
    //     finalized_block: &BeaconBlock,
    //     finalized_block_root: Hash256,
    // ) -> Result<()> {
    //     self.backend
    //         .update_finalized_root(finalized_block, finalized_block_root)
    //         .map_err(Into::into)
    // }
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
