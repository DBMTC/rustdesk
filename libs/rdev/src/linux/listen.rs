use crate::rdev::{Event, ListenError};

pub fn listen<T>(_callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    Ok(())
}
