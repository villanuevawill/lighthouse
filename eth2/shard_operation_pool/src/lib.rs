mod attestation_id;

use attestation_id::AttestationId;
use itertools::Itertools;
use parking_lot::RwLock;
use std::collections::{btree_map::Entry, hash_map, BTreeMap, HashMap, HashSet};
use std::marker::PhantomData;
use types::{
    ShardAttestation, ShardState, ChainSpec, EthSpec, Validator
};

#[derive(Default, Debug)]
pub struct ShardOperationPool<T: EthSpec + Default> {
    attestations: RwLock<HashMap<AttestationId, Vec<ShardAttestation>>>,
    _phantom: PhantomData<T>,
}

impl<T: EthSpec> ShardOperationPool<T> {
    /// Create a new operation pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an attestation into the pool, aggregating it with existing attestations if possible.
    pub fn insert_attestation(
        &self,
        attestation: ShardAttestation,
        state: &ShardState<T>,
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
    pub fn get_attestations(&self, state: &ShardState<T>, spec: &ChainSpec) -> Vec<ShardAttestation> {
        // Attestations for the current fork, which may be from the current or previous epoch.
        let current_slot = state.slot();
        let domain_bytes = AttestationId::compute_domain_bytes(epoch, state, spec);
        let reader = self.attestations.read();
        let valid_attestations = reader
            .iter()
            .filter(|(key, _)| key.domain_bytes_match(&domain_bytes))
            .flat_map(|(_, attestations)| attestations);
    }

    pub fn prune_attestations(&self, finalized_state: &ShardState<T>) {
        self.attestations.write().retain(|_, attestations| {
            attestations.first().map_or(false, |att| {
                finalized_state.current_slot() <= att.data.target_slot
            })
        });
    }
}

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

impl<T: EthSpec + Default> PartialEq for ShardOperationPool<T> {
    fn eq(&self, other: &Self) -> bool { *self.attestations.read() == *other.attestations.read()}
}
