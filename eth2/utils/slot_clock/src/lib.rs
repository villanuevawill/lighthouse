#[macro_use]
extern crate lazy_static;

mod metrics;
mod system_time_slot_clock;
mod testing_slot_clock;

pub use crate::system_time_slot_clock::{Error as SystemTimeSlotClockError, SystemTimeSlotClock};
pub use crate::testing_slot_clock::{Error as TestingSlotClockError, TestingSlotClock, ShardTestingSlotClock};
pub use metrics::scrape_for_metrics;
use std::time::Duration;
pub use types::{Slot, ShardSlot};

/// A clock that reports the current slot.
///
/// The clock is not required to be monotonically increasing and may go backwards.
pub trait SlotClock: Send + Sync + Sized {
    /// Creates a new slot clock where the first slot is `genesis_slot`, genesis occured
    /// `genesis_duration` after the `UNIX_EPOCH` and each slot is `slot_duration` apart.
    fn new(genesis_slot: Slot, genesis_duration: Duration, slot_duration: Duration) -> Self;

    /// Returns the slot at this present time.
    fn now(&self) -> Option<Slot>;

    /// Returns the duration between slots
    fn slot_duration(&self) -> Duration;

    /// Returns the duration until the next slot.
    fn duration_to_next_slot(&self) -> Option<Duration>;
}

pub trait ShardSlotClock: Send + Sync + Sized {
    type Error;

    /// Create a new `SlotClock`.
    ///
    /// Returns an Error if `slot_duration_seconds == 0`.
    fn new(genesis_slot: ShardSlot, genesis_seconds: u64, slot_duration_seconds: u64) -> Self;

    fn present_slot(&self) -> Result<Option<ShardSlot>, Self::Error>;

    fn duration_to_next_slot(&self) -> Result<Option<Duration>, Self::Error>;
}
