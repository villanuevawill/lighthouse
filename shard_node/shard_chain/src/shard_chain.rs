

pub trait ShardChainTypes {
    type Store: store::Store;
    type SlotClock: slot_clock::SlotClock;
    type LmdGhost: LmdGhost<Self::Store, Self::EthSpec>;
    type EthSpec: types::EthSpec;
}

/// Represents the "Shard Chain" component of Ethereum 2.0. It holds a reference to a parent Beacon Chain
pub struct ShardChain<T: ShardChainTypes> {
    pub spec: ChainSpec,
    /// 
    /// Persistent storage for blocks, states, etc. Typically an on-disk store, such as LevelDB.
    pub store: Arc<T::Store>,
    /// Reports the current slot, typically based upon the system clock.
    pub slot_clock: T::SlotClock,
    /// Stores all operations (e.g., transactions) that are candidates for
    /// inclusion in a block.
    pub op_pool: OperationPool<T::EthSpec>,
    /// Stores a "snapshot" of the chain at the time the head-of-the-chain block was recieved.
    canonical_head: RwLock<CheckPoint<T::EthSpec>>,
    /// The same state from `self.canonical_head`, but updated at the start of each slot with a
    /// skip slot if no block is recieved. This is effectively a cache that avoids repeating calls
    /// to `per_slot_processing`.
    state: RwLock<ShardState<T::EthSpec>>,
    /// The root of the genesis block.
    genesis_block_root: Hash256,
    /// A state-machine that is updated with information from the network and chooses a canonical
    /// head block.
    pub fork_choice: ForkChoice<T>,
}

impl<T: BeaconChainTypes> BeaconChain<T> {
    /// Instantiate a new Beacon Chain, from genesis.
    pub fn from_genesis(
        store: Arc<T::Store>,
        slot_clock: T::SlotClock,
        mut genesis_state: ShardState<T::EthSpec>,
        genesis_block: ShardBlock,
        spec: ChainSpec,
    ) -> Result<Self, Error> {
        genesis_state.build_all_caches(&spec)?;

        let state_root = genesis_state.canonical_root();
        store.put(&state_root, &genesis_state)?;

        let genesis_block_root = genesis_block.block_header().canonical_root();
        store.put(&genesis_block_root, &genesis_block)?;

        // Also store the genesis block under the `ZERO_HASH` key.
        let genesis_block_root = genesis_block.block_header().canonical_root();
        store.put(&spec.zero_hash, &genesis_block)?;

        let canonical_head = RwLock::new(CheckPoint::new(
            genesis_block.clone(),
            genesis_block_root,
            genesis_state.clone(),
            state_root,
        ));

        Ok(Self {
            spec,
            slot_clock,
            op_pool: OperationPool::new(),
            state: RwLock::new(genesis_state),
            canonical_head,
            genesis_block_root,
            fork_choice: ForkChoice::new(store.clone(), &genesis_block, genesis_block_root),
            metrics: Metrics::new()?,
            store,
        })
    }
}

