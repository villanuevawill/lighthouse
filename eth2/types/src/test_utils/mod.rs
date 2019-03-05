mod generate_deterministic_keypairs;
mod keypairs_file;
mod test_random;
mod testing_attestation_builder;
mod testing_attester_slashing_builder;
mod testing_beacon_block_builder;
mod testing_beacon_state_builder;
mod testing_deposit_builder;
mod testing_proposer_slashing_builder;
mod testing_transfer_builder;
mod testing_voluntary_exit_builder;

pub use generate_deterministic_keypairs::generate_deterministic_keypairs;
pub use keypairs_file::KeypairsFile;
pub use rand::{prng::XorShiftRng, SeedableRng};

pub mod address;
pub mod aggregate_signature;
pub mod bitfield;
pub mod hash256;
#[macro_use]
mod macros;
pub mod public_key;
pub mod secret_key;
pub mod signature;

pub trait TestRandom<T>
where
    T: RngCore,
{
    fn random_for_test(rng: &mut T) -> Self;
}

impl<T: RngCore> TestRandom<T> for bool {
    fn random_for_test(rng: &mut T) -> Self {
        (rng.next_u32() % 2) == 1
    }
}

impl<T: RngCore> TestRandom<T> for u64 {
    fn random_for_test(rng: &mut T) -> Self {
        rng.next_u64()
    }
}

impl<T: RngCore> TestRandom<T> for u32 {
    fn random_for_test(rng: &mut T) -> Self {
        rng.next_u32()
    }
}

impl<T: RngCore> TestRandom<T> for usize {
    fn random_for_test(rng: &mut T) -> Self {
        rng.next_u32() as usize
    }
}

impl<T: RngCore, U> TestRandom<T> for Vec<U>
where
    U: TestRandom<T>,
{
    fn random_for_test(rng: &mut T) -> Self {
        vec![
            <U>::random_for_test(rng),
            <U>::random_for_test(rng),
            <U>::random_for_test(rng),
        ]
    }
}
