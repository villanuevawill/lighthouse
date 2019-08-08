type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingBlock(Hash256),
    MissingState(Hash256),
    BackendError(String),
    ShardStateError(ShardStateError),
    StoreError(StoreError),
}

pub struct ForkChoice<T: ShardChainTypes> {
    backend: T::ShardLmdGhost,
    /// Used for resolving the `0x00..00` alias back to genesis.
    ///
    /// Does not necessarily need to be the _actual_ genesis, it suffices to be the finalized root
    /// whenever the struct was instantiated.
    genesis_block_root: Hash256,
}

impl<T: BeaconChainTypes> ForkChoice<T> {
    /// Instantiate a new fork chooser.
    ///
    /// "Genesis" does not necessarily need to be the absolute genesis, it can be some finalized
    /// block.
    pub fn new(
        store: Arc<T::Store>,
        genesis_block: &ShardBlock,
        genesis_block_root: Hash256,
    ) -> Self {
        Self {
            backend: T::ShardLmdGhost::new(store, genesis_block, genesis_block_root),
            genesis_block_root,
        }
    }

    // general pseudocode here
    pub fn find_head(&self, chain: &ShardChain<T>) -> Result<Hash256> {
        let beacon_state = chain.get_beacon_state();
        let finalized_epoch = beacon_state.finalized_epoch;
        let start_block_root = chain.get_crosslink(finalized_epoch);
        let start_block_slot = chain.get_block(start_block_root).slot;

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
    pub fn process_block(
        &self,
        beacon_state: &BeaconState,
        block: &ShardBlock,
        block_root: Hash256,
    ) -> Result<()> {
        // Note: we never count the block as a latest message, only attestations.
        //
        // I (Paul H) do not have an explicit reference to this, but I derive it from this
        // document:
        //
        // https://github.com/ethereum/eth2.0-specs/blob/v0.7.0/specs/core/0_fork-choice.md
        for attestation in &block.body.attestations {
            self.process_attestation_from_block(beacon_state, attestation, block)?;
        }

        self.backend.process_block(block, block_root)?;

        Ok(())
    }

    fn process_attestation_from_block(
        &self,
        beacon_state: &BeaconState<T::EthSpec>,
        attestation: &Attestation,
        block: &ShardBlock
    ) -> Result<()> {
        // Note: `get_attesting_indices_unsorted` requires that the beacon state caches be built.
        let validator_indices = get_shard_attesting_indices_unsorted(
            block.slot,
            beacon_state,
            &attestation.data,
            &attestation.aggregation_bitfield,
        )?;

        let block_hash = attestation.data.target_root;

        if block_hash != Hash256::zero() {
            for validator_index in validator_indices {
                self.backend
                    .process_attestation(validator_index, block_hash, block.slot)?;
            }
        }

        Ok(())
    }

    /// Inform the fork choice that the given block (and corresponding root) have been finalized so
    /// it may prune it's storage.
    ///
    /// `finalized_block_root` must be the root of `finalized_block`.
    pub fn process_finalization(
        &self,
        finalized_block: &BeaconBlock,
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

impl From<StoreError> for Error {
    fn from(e: StoreError) -> Error {
        Error::StoreError(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::BackendError(e)
    }
}
