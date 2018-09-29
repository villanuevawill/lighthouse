extern crate ssz;

use self::ssz::{
    SszStream,
};
use std::sync::Arc;
use super::{
    validate_ssz_block,
    SszBlockValidationError,
    BlockStatus,
    AttesterMap,
    ProposerMap,
};
use super::db::stores::{
    BlockStore,
    PoWChainStore,
    ValidatorStore,
};
use super::db::{
    MemoryDB,
};
use super::utils::hash::canonical_hash;
use super::utils::types::{
    Hash256,
    Bitfield,
};
use super::SszBlock;
use super::super::Block;
use super::super::attestation_record::AttestationRecord;
use super::super::super::bls::{
    Keypair,
    Signature,
    AggregateSignature,
};

pub struct TestStore {
    pub db: Arc<MemoryDB>,
    pub block: Arc<BlockStore<MemoryDB>>,
    pub pow_chain: Arc<PoWChainStore<MemoryDB>>,
    pub validator: Arc<ValidatorStore<MemoryDB>>,
}

impl TestStore {
    pub fn new() -> Self {
        let db = Arc::new(MemoryDB::open());
        let block = Arc::new(BlockStore::new(db.clone()));
        let pow_chain = Arc::new(PoWChainStore::new(db.clone()));
        let validator = Arc::new(ValidatorStore::new(db.clone()));
        Self {
            db,
            block,
            pow_chain,
            validator,
        }
    }
}

#[derive(Debug)]
pub struct TestParams {
    pub total_validators: usize,
    pub cycle_length: u8,
    pub shard_count: u16,
    pub shards_per_slot: u16,
    pub validators_per_shard: usize,
    pub block_slot: u64,
    pub attestations_justified_slot: u64,
}
type ParentHashes = Vec<Hash256>;

/// Setup for a block validation function, without actually executing the
/// block validation function.
pub fn setup_block_validation_scenario(params: &TestParams)
    -> (Block, ParentHashes, AttesterMap, ProposerMap, TestStore)
{
    let stores = TestStore::new();

    let cycle_length = params.cycle_length;
    let shards_per_slot = params.shards_per_slot;
    let validators_per_shard = params.validators_per_shard;
    let block_slot = params.block_slot;
    let attestations_justified_slot = params.attestations_justified_slot;

    let parent_hashes: Vec<Hash256> = (0..(cycle_length * 2))
        .map(|i| Hash256::from(i as u64))
        .collect();
    let parent_hash = Hash256::from("parent_hash".as_bytes());
    let randao_reveal = Hash256::from("randao_reveal".as_bytes());
    let justified_block_hash = Hash256::from("justified_hash".as_bytes());
    let pow_chain_ref = Hash256::from("pow_chain".as_bytes());
    let active_state_root = Hash256::from("active_state".as_bytes());
    let crystallized_state_root = Hash256::from("cry_state".as_bytes());
    let shard_block_hash = Hash256::from("shard_block_hash".as_bytes());

    stores.pow_chain.put_block_hash(pow_chain_ref.as_ref()).unwrap();
    stores.block.put_block(justified_block_hash.as_ref(), &vec![42]).unwrap();
    stores.block.put_block(parent_hash.as_ref(), &vec![42]).unwrap();

    let validator_index: usize = 0;
    let proposer_map = {
        let mut proposer_map = ProposerMap::new();
        proposer_map.insert(block_slot, validator_index);
        proposer_map
    };
    /*
    let attestation_slot = block_slot - 1;
    let (attester_map, attestations, _keypairs) =
        generate_attestations_for_slot(
            attestation_slot,
            block_slot,
            shards_per_slot,
            validators_per_shard,
            cycle_length,
            &parent_hashes,
            &Hash256::from("shard_hash".as_bytes()),
            &justified_block_hash,
            attestations_justified_slot,
            &stores);
    */
    let (attester_map, attestations, _keypairs) = {
        let mut i = 0;
        let attestation_slot = block_slot - 1;
        let mut attester_map = AttesterMap::new();
        let mut attestations = vec![];
        let mut keypairs = vec![];
        for shard in 0..shards_per_slot {
            let mut attesters = vec![];
            let mut attester_bitfield = Bitfield::new();
            let mut aggregate_sig = AggregateSignature::new();

            let parent_hashes_slice = {
                let distance: usize = (block_slot - attestation_slot) as usize;
                let last: usize = parent_hashes.len() - distance;
                let first: usize = last - usize::from(cycle_length);
                &parent_hashes[first..last]
            };

            let attestation_message = {
                let mut stream = SszStream::new();
                stream.append(&attestation_slot);
                stream.append_vec(&parent_hashes_slice.to_vec());
                stream.append(&shard);
                stream.append(&shard_block_hash);
                stream.append(&attestations_justified_slot);
                let bytes = stream.drain();
                canonical_hash(&bytes)
            };



            for attestation_index in 0..validators_per_shard {
               /*
                * Add the attester to the attestation indices for this shard.
                */
               attesters.push(i);
               /*
                * Set the voters bit on the bitfield to true.
                */
               attester_bitfield.set_bit(attestation_index, true);
               /*
                * Generate a random keypair for this validatior and clone it into the
                * list of keypairs.
                */
               let keypair = Keypair::random();
               keypairs.push(keypair.clone());
               /*
                * Store the validators public key in the database.
                */
               stores.validator.put_public_key_by_index(i, &keypair.pk).unwrap();
               /*
                * Generate a new signature and aggregate it on the rolling signature.
                */
               let sig = Signature::new(&attestation_message, &keypair.sk);
               aggregate_sig.add(&sig);
               /*
                * Increment the validator counter to monotonically assign validators.
                */
               i += 1;
            }

            attester_map.insert((attestation_slot, shard), attesters);
            let attestation = AttestationRecord {
                slot: attestation_slot,
                shard_id: shard,
                oblique_parent_hashes: vec![],
                shard_block_hash,
                attester_bitfield,
                justified_slot: attestations_justified_slot,
                justified_block_hash,
                aggregate_sig,
            };
            attestations.push(attestation);
        }
        (attester_map, attestations, keypairs)
    };

    let block = Block {
        parent_hash,
        slot_number: block_slot,
        randao_reveal,
        attestations,
        pow_chain_ref,
        active_state_root,
        crystallized_state_root,
    };

    (block,
     parent_hashes,
     attester_map,
     proposer_map,
     stores)
}

/// Helper function to take some Block and SSZ serialize it.
pub fn serialize_block(b: &Block) -> Vec<u8> {
    let mut stream = SszStream::new();
    stream.append(b);
    stream.drain()
}

/// Setup and run a block validation scenario, given some parameters.
///
/// Returns the Result returned from the block validation function.
pub fn run_block_validation_scenario<F>(
    validation_slot: u64,
    validation_last_justified_slot: u64,
    params: &TestParams,
    mutator_func: F)
    -> Result<(BlockStatus, Option<Block>), SszBlockValidationError>
    where F: FnOnce(Block, AttesterMap, ProposerMap, TestStore)
                -> (Block, AttesterMap, ProposerMap, TestStore)
{
    let (block,
     parent_hashes,
     attester_map,
     proposer_map,
     stores) = setup_block_validation_scenario(&params);

    let (block,
         attester_map,
         proposer_map,
         stores) = mutator_func(block, attester_map, proposer_map, stores);

    let ssz_bytes = serialize_block(&block);
    let ssz_block = SszBlock::from_slice(&ssz_bytes[..])
        .unwrap();

    validate_ssz_block(
        &ssz_block,
        validation_slot,
        params.cycle_length,
        validation_last_justified_slot,
        &Arc::new(parent_hashes),
        &Arc::new(proposer_map),
        &Arc::new(attester_map),
        &stores.block.clone(),
        &stores.validator.clone(),
        &stores.pow_chain.clone())
}

fn get_simple_params() -> TestParams {
    let validators_per_shard: usize = 5;
    let cycle_length: u8 = 2;
    let shard_count: u16 = 4;
    let shards_per_slot: u16 = shard_count / u16::from(cycle_length);
    let total_validators: usize = validators_per_shard * shard_count as usize;
    let block_slot = u64::from(cycle_length) * 10000;
    let attestations_justified_slot = block_slot - u64::from(cycle_length);

    TestParams {
        total_validators,
        cycle_length,
        shard_count,
        shards_per_slot,
        validators_per_shard,
        block_slot,
        attestations_justified_slot,
    }
}

#[test]
fn test_block_validation_simple_scenario_valid_in_canonical_chain() {
    let params = get_simple_params();

    let validation_slot = params.block_slot;
    let validation_last_justified_slot = params.attestations_justified_slot;

    let no_mutate = |block, attester_map, proposer_map, stores| {
        (block, attester_map, proposer_map, stores)
    };

    let status = run_block_validation_scenario(
        validation_slot,
        validation_last_justified_slot,
        &params,
        no_mutate);

    assert_eq!(status.unwrap().0, BlockStatus::NewBlockInCanonicalChain);
}

#[test]
fn test_block_validation_simple_scenario_valid_not_in_canonical_chain() {
    let params = get_simple_params();

    let validation_slot = params.block_slot;
    let validation_last_justified_slot = params.attestations_justified_slot;

    let no_mutate = |mut block: Block, attester_map, proposer_map, stores| {
        block.parent_hash = Hash256::from("not in canonical chain".as_bytes());
        (block, attester_map, proposer_map, stores)
    };

    let status = run_block_validation_scenario(
        validation_slot,
        validation_last_justified_slot,
        &params,
        no_mutate);

    assert_eq!(status.unwrap().0, BlockStatus::NewBlockInForkChain);
}

#[test]
fn test_block_validation_simple_scenario_invalid_2nd_attestation() {
    let params = get_simple_params();

    let validation_slot = params.block_slot;
    let validation_last_justified_slot = params.attestations_justified_slot;

    let mutator = |mut block: Block, attester_map, proposer_map, stores| {
        /*
         * Set the second attestaion record to have an invalid signature.
         */
        block.attestations[1].aggregate_sig = AggregateSignature::new();
        (block, attester_map, proposer_map, stores)
    };

    let status = run_block_validation_scenario(
        validation_slot,
        validation_last_justified_slot,
        &params,
        mutator);

    assert_eq!(status, Err(SszBlockValidationError::AttestationSignatureFailed));
}
