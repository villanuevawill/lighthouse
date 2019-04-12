use super::PublicKey;
use bls_aggregates::AggregatePublicKey as RawAggregatePublicKey;

/// A BLS aggregate public key.
///
/// This struct is a wrapper upon a base type and provides helper functions (e.g., SSZ
/// serialization).
#[derive(Debug, Clone, Default)]
pub struct AggregatePublicKey(RawAggregatePublicKey);

impl AggregatePublicKey {
    pub fn new() -> Self {
        AggregatePublicKey(RawAggregatePublicKey::new())
    }

    pub fn add(&mut self, public_key: &PublicKey) {
        self.0.add(public_key.as_raw())
    }

    pub fn add_many(&mut self, pubkeys: &[&PublicKey]) {
        let pubkeys: Vec<_> = pubkeys.iter().map(|pk| pk.as_raw()).collect();
        self.0.add_many(&pubkeys)
    }

    /// Returns the underlying public key.
    pub fn as_raw(&self) -> &RawAggregatePublicKey {
        &self.0
    }
}
