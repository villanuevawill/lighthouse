use super::*;
use ssz::{Decode, DecodeError};

fn get_block_bytes<T: Store>(store: &T, root: Hash256) -> Result<Option<Vec<u8>>, Error> {
    store.get_bytes(ShardBlock::db_column().into(), &root[..])
}

fn read_slot_from_block_bytes(bytes: &[u8]) -> Result<ShardSlot, DecodeError> {
    let end = std::cmp::min(ShardSlot::ssz_fixed_len(), bytes.len());

    ShardSlot::from_ssz_bytes(&bytes[0..end])
}

fn read_parent_root_from_block_bytes(bytes: &[u8]) -> Result<Hash256, DecodeError> {
    let previous_bytes = ShardSlot::ssz_fixed_len();
    let slice = bytes
        .get(previous_bytes..previous_bytes + Hash256::ssz_fixed_len())
        .ok_or_else(|| DecodeError::BytesInvalid("Not enough bytes.".to_string()))?;

    Hash256::from_ssz_bytes(slice)
}

pub fn get_block_at_preceeding_slot<T: Store>(
    store: &T,
    slot: ShardSlot,
    start_root: Hash256,
) -> Result<Option<(Hash256, ShardBlock)>, Error> {
    Ok(match get_at_preceeding_slot(store, slot, start_root)? {
        Some((hash, bytes)) => Some((hash, ShardBlock::from_ssz_bytes(&bytes)?)),
        None => None,
    })
}

fn get_at_preceeding_slot<T: Store>(
    store: &T,
    slot: ShardSlot,
    mut root: Hash256,
) -> Result<Option<(Hash256, Vec<u8>)>, Error> {
    loop {
        if let Some(bytes) = get_block_bytes(store, root)? {
            let this_slot = read_slot_from_block_bytes(&bytes)?;

            if this_slot == slot {
                break Ok(Some((root, bytes)));
            } else if this_slot < slot {
                break Ok(None);
            } else {
                root = read_parent_root_from_block_bytes(&bytes)?;
            }
        } else {
            break Ok(None);
        }
    }
}
