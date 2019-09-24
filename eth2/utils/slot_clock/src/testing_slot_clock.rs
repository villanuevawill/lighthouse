use super::{SlotClock, ShardSlotClock};
use std::sync::RwLock;
use std::time::Duration;
use types::{Slot, ShardSlot};

/// A slot clock where the slot is manually set instead of being determined by the system time.
///
/// Useful for testing scenarios.
pub struct TestingSlotClock {
    slot: RwLock<Slot>,
}

pub struct ShardTestingSlotClock {
    slot: RwLock<ShardSlot>,
}

impl TestingSlotClock {
    pub fn set_slot(&self, slot: u64) {
        *self.slot.write().expect("TestingSlotClock poisoned.") = Slot::from(slot);
    }

    pub fn advance_slot(&self) {
        self.set_slot(self.now().unwrap().as_u64() + 1)
    }
}


impl ShardTestingSlotClock {
    pub fn set_slot(&self, slot: u64) {
        *self.slot.write().expect("TestingSlotClock poisoned.") = ShardSlot::from(slot);
    }

    pub fn advance_slot(&self) {
        self.set_slot(self.present_slot().unwrap().unwrap().as_u64() + 1)
    }
}

impl SlotClock for TestingSlotClock {
    fn new(genesis_slot: Slot, _genesis_duration: Duration, _slot_duration: Duration) -> Self {
        TestingSlotClock {
            slot: RwLock::new(genesis_slot),
        }
    }

    fn now(&self) -> Option<Slot> {
        let slot = *self.slot.read().expect("TestingSlotClock poisoned.");
        Some(slot)
    }

    /// Always returns a duration of 1 second.
    fn duration_to_next_slot(&self) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    /// Always returns a slot duration of 0 seconds.
    fn slot_duration(&self) -> Duration {
        Duration::from_secs(0)
    }
}

impl ShardSlotClock for ShardTestingSlotClock {
    type Error = Error;

    /// Create a new `TestingSlotClock` at `genesis_slot`.
    fn new(genesis_slot: ShardSlot, _genesis_seconds: u64, _slot_duration_seconds: u64) -> Self {
        ShardTestingSlotClock {
            slot: RwLock::new(genesis_slot),
        }
    }

    fn present_slot(&self) -> Result<Option<ShardSlot>, Error> {
        let slot = *self.slot.read().expect("TestingSlotClock poisoned.");
        Ok(Some(slot))
    }

    /// Always returns a duration of 1 second.
    fn duration_to_next_slot(&self) -> Result<Option<Duration>, Error> {
        Ok(Some(Duration::from_secs(1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_now() {
        let null = Duration::from_secs(0);

        let clock = TestingSlotClock::new(Slot::new(10), null, null);
        assert_eq!(clock.now(), Some(Slot::new(10)));
        clock.set_slot(123);
        assert_eq!(clock.now(), Some(Slot::new(123)));
    }
}
