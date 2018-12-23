use rand::RngCore;

pub mod aggregate_signature;
pub mod bitfield;
pub mod hash256;
pub mod signature;
pub mod secret_key;
pub mod public_key;

pub trait TestRandom<T>
where T: RngCore
{
    fn random_for_test(rng: &mut T) -> Self;
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

impl<T: RngCore, U> TestRandom<T> for Vec<U>
where U: TestRandom<T>
{
    fn random_for_test(rng: &mut T) -> Self {
        vec![
            <U>::random_for_test(rng),
            <U>::random_for_test(rng),
            <U>::random_for_test(rng),
        ]
    }
}
