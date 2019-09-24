use crate::*;
use ssz::{Decode, Encode};

impl StoreItem for ShardBlock {
    fn db_column() -> DBColumn {
        DBColumn::ShardBlock
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &mut [u8]) -> Result<Self, Error> {
        Self::from_ssz_bytes(bytes).map_err(Into::into)
    }
}

impl<T: ShardSpec> StoreItem for ShardState<T> {
    fn db_column() -> DBColumn {
        DBColumn::ShardState
    }

    fn as_store_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }

    fn from_store_bytes(bytes: &mut [u8]) -> Result<Self, Error> {
        Self::from_ssz_bytes(bytes).map_err(Into::into)
    }
}
