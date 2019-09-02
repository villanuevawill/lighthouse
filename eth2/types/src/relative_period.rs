use crate::*;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RelativePeriod {
    /// The prior period.
    Previous,
    /// The current period.
    Current,
    /// The next period.
    Next,
}
