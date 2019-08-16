mod attestation;
mod attestation_id;
mod max_cover;
mod persistence;

pub use persistence::PersistedOperationPool;

use attestation::{earliest_attestation_validators, AttMaxCover};
use attestation_id::AttestationId;
use itertools::Itertools;
use max_cover::maximum_cover;
use parking_lot::RwLock;
use state_processing::per_block_processing::{
    get_slashable_indices_modular, validate_attestation,
    validate_attestation_time_independent_only, verify_attester_slashing, verify_exit,
    verify_exit_time_independent_only, verify_proposer_slashing, verify_transfer,
    verify_transfer_time_independent_only,
};
use std::collections::{btree_map::Entry, hash_map, BTreeMap, HashMap, HashSet};
use std::marker::PhantomData;
use types::{
    ShardAttestation, BeaconState, ShardState, ChainSpec, EthSpec, Validator
};

#[derive(Default, Debug)]
pub struct OperationPool<T: EthSpec + Default> {
    /// Map from attestation ID (see below) to vectors of attestations.
    attestations: RwLock<HashMap<AttestationId, Vec<ShardAttestation>>>,
    // NOTE: We assume that there is only one deposit per index
    // because the Eth1 data is updated (at most) once per epoch,
    // and the spec doesn't seem to accomodate for re-orgs on a time-frame
    // longer than an epoch
    _phantom: PhantomData<T>,
}

impl<T: EthSpec> OperationPool<T> {
    /// Create a new operation pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an attestation into the pool, aggregating it with existing attestations if possible.
    pub fn insert_attestation(
        &self,
        attestation: Shardttestation,
        state: &BeaconState<T>,
        shard_state: &ShardState<T>,
        spec: &ChainSpec,
    ) -> () {
        let id = AttestationId::from_data(&attestation.data, state, spec);

        // Take a write lock on the attestations map.
        let mut attestations = self.attestations.write();

        let existing_attestations = match attestations.entry(id) {
            hash_map::Entry::Vacant(entry) => {
                entry.insert(vec![attestation]);
                return Ok(());
            }
            hash_map::Entry::Occupied(entry) => entry.into_mut(),
        };

        let mut aggregated = false;
        for existing_attestation in existing_attestations.iter_mut() {
            if existing_attestation.signers_disjoint_from(&attestation) {
                existing_attestation.aggregate(&attestation);
                aggregated = true;
            } else if *existing_attestation == attestation {
                aggregated = true;
            }
        }

        if !aggregated {
            existing_attestations.push(attestation);
        }

        Ok(())
    }

    /// Total number of attestations in the pool, including attestations for the same data.
    pub fn num_attestations(&self) -> usize {
        self.attestations.read().values().map(Vec::len).sum()
    }

    /// Get a list of attestations for inclusion in a block.
    pub fn get_attestations(&self, state: &ShardState<T>, spec: &ChainSpec) -> Vec<Attestation> {
        // Attestations for the current fork, which may be from the current or previous epoch.
        let current_slot = state.slot();
        let domain_bytes = AttestationId::compute_domain_bytes(epoch, state, spec);
        let reader = self.attestations.read();
        let valid_attestations = reader
            .iter()
            .filter(|(key, _)| key.domain_bytes_match(&domain_bytes))
            .flat_map(|(_, attestations)| attestations)
            // remove valid check for now...
            .map(|att| AttMaxCover::new(att, earliest_attestation_validators(att, state)));

        maximum_cover(valid_attestations, spec.max_attestations as usize)
    }

    /// Remove attestations which are too old to be included in a block.
    pub fn prune_attestations(&self, finalized_state: &BeaconState<T>) {
        // We know we can include an attestation if:
        // state.slot <= attestation_slot + SLOTS_PER_EPOCH
        // We approximate this check using the attestation's epoch, to avoid computing
        // the slot or relying on the committee cache of the finalized state.
        self.attestations.write().retain(|_, attestations| {
            // All the attestations in this bucket have the same data, so we only need to
            // check the first one.
            attestations.first().map_or(false, |att| {
                finalized_state.current_epoch() <= att.data.target_epoch + 1
            })
        });
    }
}

/// Filter up to a maximum number of operations out of an iterator.
fn filter_limit_operations<'a, T: 'a, I, F>(operations: I, filter: F, limit: u64) -> Vec<T>
where
    I: IntoIterator<Item = &'a T>,
    F: Fn(&T) -> bool,
    T: Clone,
{
    operations
        .into_iter()
        .filter(|x| filter(*x))
        .take(limit as usize)
        .cloned()
        .collect()
}

/// Compare two operation pools.
impl<T: EthSpec + Default> PartialEq for OperationPool<T> {
    fn eq(&self, other: &Self) -> bool { *self.attestations.read() == *other.attestations.read()}
}
