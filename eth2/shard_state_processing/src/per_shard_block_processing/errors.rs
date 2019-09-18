use types::*;

// macro_rules! impl_from_beacon_state_error {
//     ($type: ident) => {
//         impl From<BeaconStateError> for $type {
//             fn from(e: BeaconStateError) -> $type {
//                 $type::BeaconStateError(e)
//             }
//         }
//     };
// }
// 
// macro_rules! impl_into_with_index_with_beacon_error {
//     ($error_type: ident, $invalid_type: ident) => {
//         impl IntoWithIndex<BlockProcessingError> for $error_type {
//             fn into_with_index(self, i: usize) -> BlockProcessingError {
//                 match self {
//                     $error_type::Invalid(e) => {
//                         BlockProcessingError::Invalid(BlockInvalid::$invalid_type(i, e))
//                     }
//                     $error_type::BeaconStateError(e) => BlockProcessingError::BeaconStateError(e),
//                 }
//             }
//         }
//     };
// }
// 
// macro_rules! impl_into_with_index_without_beacon_error {
//     ($error_type: ident, $invalid_type: ident) => {
//         impl IntoWithIndex<BlockProcessingError> for $error_type {
//             fn into_with_index(self, i: usize) -> BlockProcessingError {
//                 match self {
//                     $error_type::Invalid(e) => {
//                         BlockProcessingError::Invalid(BlockInvalid::$invalid_type(i, e))
//                     }
//                 }
//             }
//         }
//     };
// }

/// A conversion that consumes `self` and adds an `index` variable to resulting struct.
///
/// Used here to allow converting an error into an upstream error that points to the object that
/// caused the error. For example, pointing to the index of an attestation that caused the
/// `AttestationInvalid` error.
pub trait IntoWithIndex<T>: Sized {
    fn into_with_index(self, index: usize) -> T;
}

/*
 * Block Validation
 */

// /// The object is invalid or validation failed.
// #[derive(Debug, PartialEq)]
// pub enum BlockProcessingError {
//     /// Validation completed successfully and the object is invalid.
//     Invalid(BlockInvalid),
//     /// Encountered a `BeaconStateError` whilst attempting to determine validity.
//     BeaconStateError(BeaconStateError),
// }

// impl_from_beacon_state_error!(BlockProcessingError);

/// Describes why an object is invalid.
#[derive(Debug, PartialEq)]
pub enum Error {
    BlockProcessingError,
    StateSlotMismatch,
    ParentBlockRootMismatch {
        state: Hash256,
        block: Hash256,
    },
}

// impl Into<BlockProcessingError> for BlockInvalid {
//     fn into(self) -> BlockProcessingError {
//         BlockProcessingError::Invalid(self)
//     }
// }

// /*
//  * Attestation Validation
//  */
// 
// /// The object is invalid or validation failed.
// #[derive(Debug, PartialEq)]
// pub enum AttestationValidationError {
//     /// Validation completed successfully and the object is invalid.
//     Invalid(AttestationInvalid),
//     /// Encountered a `BeaconStateError` whilst attempting to determine validity.
//     BeaconStateError(BeaconStateError),
// }
// 
// /// Describes why an object is invalid.
// #[derive(Debug, PartialEq)]
// pub enum AttestationInvalid {
//     /// Attestation references a pre-genesis slot.
//     PreGenesis { genesis: Slot, attestation: Slot },
//     /// Attestation included before the inclusion delay.
//     IncludedTooEarly {
//         state: Slot,
//         delay: u64,
//         attestation: Slot,
//     },
//     /// Attestation slot is too far in the past to be included in a block.
//     IncludedTooLate { state: Slot, attestation: Slot },
//     /// Attestation target epoch does not match the current or previous epoch.
//     BadTargetEpoch,
//     /// Attestation justified epoch does not match the states current or previous justified epoch.
//     ///
//     /// `is_current` is `true` if the attestation was compared to the
//     /// `state.current_justified_epoch`, `false` if compared to `state.previous_justified_epoch`.
//     WrongJustifiedEpoch {
//         state: Epoch,
//         attestation: Epoch,
//         is_current: bool,
//     },
//     /// Attestation justified epoch root does not match root known to the state.
//     ///
//     /// `is_current` is `true` if the attestation was compared to the
//     /// `state.current_justified_epoch`, `false` if compared to `state.previous_justified_epoch`.
//     WrongJustifiedRoot {
//         state: Hash256,
//         attestation: Hash256,
//         is_current: bool,
//     },
//     /// Attestation crosslink root does not match the state crosslink root for the attestations
//     /// slot.
//     BadPreviousCrosslink,
//     /// The custody bitfield has some bits set `true`. This is not allowed in phase 0.
//     CustodyBitfieldHasSetBits,
//     /// There are no set bits on the attestation -- an attestation must be signed by at least one
//     /// validator.
//     AggregationBitfieldIsEmpty,
//     /// The custody bitfield length is not the smallest possible size to represent the committee.
//     BadCustodyBitfieldLength {
//         committee_len: usize,
//         bitfield_len: usize,
//     },
//     /// The aggregation bitfield length is not the smallest possible size to represent the committee.
//     BadAggregationBitfieldLength {
//         committee_len: usize,
//         bitfield_len: usize,
//     },
//     /// There was no known committee in this `epoch` for the given shard and slot.
//     NoCommitteeForShard { shard: u64, slot: Slot },
//     /// The validator index was unknown.
//     UnknownValidator(u64),
//     /// The attestation signature verification failed.
//     BadSignature,
//     /// The shard block root was not set to zero. This is a phase 0 requirement.
//     ShardBlockRootNotZero,
// }

// impl_from_beacon_state_error!(AttestationValidationError);
// impl_into_with_index_with_beacon_error!(AttestationValidationError, AttestationInvalid);
 
// impl From<IndexedAttestationValidationError> for AttestationValidationError {
//     fn from(err: IndexedAttestationValidationError) -> Self {
//         let IndexedAttestationValidationError::Invalid(e) = err;
//         AttestationValidationError::Invalid(AttestationInvalid::BadIndexedAttestation(e))
//     }
// }

