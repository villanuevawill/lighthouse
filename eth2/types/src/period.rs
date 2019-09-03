use serde_derive::{Deserialize, Serialize};
use ssz::{ssz_encode, Decode, DecodeError, Encode};
use std::fmt;
use std::iter::Iterator;
use slog;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};
use rand::RngCore;
use crate::test_utils::TestRandom;
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
