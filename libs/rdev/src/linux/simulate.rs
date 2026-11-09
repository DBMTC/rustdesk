use crate::rdev::{EventType, SimulateError};

pub fn simulate(_event_type: &EventType) -> Result<(), SimulateError> {
    Ok(())
}

pub fn simulate_char(_chr: char, _down: bool) -> Result<(), SimulateError> {
    Ok(())
}

pub fn simulate_unicode(_chr: u32) -> Result<(), SimulateError> {
    Ok(())
}
