use crate::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    PeriodTooLow { base: Period, other: Period },
    PeriodTooHigh { base: Period, other: Period },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RelativePeriod {
    /// The prior period.
    Previous,
    /// The current period.
    Current,
    /// The next period.
    Next,
}

impl RelativePeriod {
    pub fn into_period(self, base: Period) -> Period {
        match self {
            // Due to saturating nature of epoch, check for current first.
            RelativePeriod::Current => base,
            RelativePeriod::Previous => base - 1,
            RelativePeriod::Next => base + 1,
        }
    }

    pub fn from_period(base: Period, other: Period) -> Result<Self, Error> {
        if other == base {
            Ok(RelativePeriod::Current)
        } else if other == base - 1 {
            Ok(RelativePeriod::Previous)
        } else if other == base + 1 {
            Ok(RelativePeriod::Next)
        } else if other < base {
            Err(Error::PeriodTooLow { base, other })
        } else {
            Err(Error::PeriodTooHigh { base, other })
        }
    }
}
