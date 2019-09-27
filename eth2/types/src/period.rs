use crate::test_utils::TestRandom;
use rand::RngCore;
use serde_derive::{Deserialize, Serialize};
use slog;
use ssz::{ssz_encode, Decode, DecodeError, Encode};
use std::cmp::{Ord, Ordering};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, Sub, SubAssign};

#[derive(Eq, Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Period(u64);

impl_common!(Period);

impl Period {
    pub fn new(period: u64) -> Period {
        Period(period)
    }

    pub fn max_value() -> Period {
        Period(u64::max_value())
    }
}
