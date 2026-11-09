use crate::{Event, GrabError};

pub fn disable_grab() {}

pub fn enable_grab() {}

pub fn exit_grab_listen() {}

pub fn is_grabbed() -> bool {
    false
}

pub fn start_grab_listen<T>(_callback: T) -> Result<(), GrabError>
where
    T: Fn(Event) -> Option<Event> + Send + 'static,
{
    Ok(())
}
