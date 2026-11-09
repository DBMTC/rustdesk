use crate::rdev::DisplayError;

pub fn display_size() -> Result<(u64, u64), DisplayError> {
    Ok((1920, 1080))
}
