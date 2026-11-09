mod rdev;
pub use crate::rdev::{
    Button, DisplayError, Event, EventType, GrabCallback, GrabError, Key, KeyCode, KeyboardState,
    ListenError, RawKey, SimulateError,
};

mod keycodes;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

mod codes_conv;

pub use crate::codes_conv::*;

pub use keycodes::android::{
    code_from_key as android_keycode_from_key, key_from_code as android_key_from_code,
};
pub use keycodes::linux::{
    code_from_key as linux_keycode_from_key, key_from_code as linux_key_from_code,
};
pub use keycodes::macos::{
    code_from_key as macos_keycode_from_key, key_from_code as macos_key_from_code,
};
pub use keycodes::usb_hid::{
    code_from_key as usb_hid_keycode_from_key, key_from_code as usb_hid_key_from_code,
};
pub use keycodes::windows::{
    code_from_key as win_code_from_key, code_from_key as win_keycode_from_key, get_win_codes,
    get_win_key, key_from_code as win_key_from_keycode, key_from_scancode as win_key_from_scancode,
    scancode_from_key as win_scancode_from_key,
};
pub use keycodes::chrome::{
    code_from_key as chrome_keycode_from_key, key_from_code as chrome_key_from_code,
};

#[cfg(target_os = "macos")]
pub use crate::keycodes::macos::{code_from_key, key_from_code, virtual_keycodes::*};
#[cfg(target_os = "macos")]
use crate::macos::{display_size as _display_size, listen as _listen, simulate as _simulate};
#[cfg(target_os = "macos")]
pub use crate::macos::{set_is_main_thread, Keyboard, VirtualInput};
#[cfg(target_os = "macos")]
pub use core_graphics::{event::CGEventTapLocation, event_source::CGEventSourceStateID};

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::keycodes::linux::{code_from_key, key_from_code};
#[cfg(target_os = "linux")]
use crate::linux::{display_size as _display_size, listen as _listen, simulate as _simulate};
#[cfg(target_os = "linux")]
pub use crate::linux::{simulate_char, simulate_unicode, Keyboard};

#[cfg(target_os = "windows")]
pub use crate::keycodes::windows::key_from_scancode;
#[cfg(target_os = "windows")]
pub use crate::windows::{
    display_size as _display_size, get_modifier, listen as _listen, set_modifier,
    simulate as _simulate, simulate_char, simulate_code, simulate_key_unicode, simulate_unicode,
    simulate_unistr, vk_to_scancode, Keyboard,
};

pub use crate::rdev::UnicodeInfo;

/// Listening to global events. Caveat: On MacOS, you require the listen
/// loop needs to be the primary app (no fork before) and need to have accessibility
/// settings enabled.
///
/// ```no_run
/// use rdev::{listen, Event};
///
/// fn callback(event: Event) {
///     println!("My callback {:?}", event);
///     match event.name{
///         Some(string) => println!("User wrote {:?}", string),
///         None => ()
///     }
/// }
/// fn main(){
///     // This will block.
///     if let Err(error) = listen(callback) {
///         println!("Error: {:?}", error)
///     }
/// }
/// ```
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    _listen(callback)
}

/// Sending some events
///
/// ```no_run
/// use rdev::{simulate, Button, EventType, Key, SimulateError};
/// use std::{thread, time};
///
/// fn send(event_type: &EventType) {
///     let delay = time::Duration::from_millis(20);
///     match simulate(event_type) {
///         Ok(()) => (),
///         Err(SimulateError) => {
///             println!("We could not send {:?}", event_type);
///         }
///     }
///     // Let ths OS catchup (at least MacOS)
///     thread::sleep(delay);
/// }
///
/// fn my_shortcut() {
///     send(&EventType::KeyPress(Key::KeyS));
///     send(&EventType::KeyRelease(Key::KeyS));
///
///     send(&EventType::MouseMove { x: 0.0, y: 0.0 });
///     send(&EventType::MouseMove { x: 400.0, y: 400.0 });
///     send(&EventType::ButtonPress(Button::Left));
///     send(&EventType::ButtonRelease(Button::Right));
///     send(&EventType::Wheel {
///         delta_x: 0,
///         delta_y: 1,
///     });
/// }
/// ```
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    _simulate(event_type)
}

/// Returns the size in pixels of the main screen.
/// This is useful to use with x, y from MouseMove Event.
///
/// ```no_run
/// use rdev::{display_size};
///
/// let (w, h) = display_size().unwrap();
/// println!("My screen size : {:?}x{:?}", w, h);
/// ```
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn display_size() -> Result<(u64, u64), DisplayError> {
    _display_size()
}

#[cfg(target_os = "linux")]
pub use crate::linux::{
    disable_grab, enable_grab, exit_grab_listen, is_grabbed, start_grab_listen,
};
#[cfg(target_os = "macos")]
pub use crate::macos::set_keyboard_extra_info;
#[cfg(target_os = "macos")]
pub use crate::macos::set_mouse_extra_info;
#[cfg(target_os = "macos")]
pub use crate::macos::{exit_grab, grab as _grab, is_grabbed};
#[cfg(target_os = "windows")]
pub use crate::windows::set_keyboard_extra_info;
#[cfg(target_os = "windows")]
pub use crate::windows::set_mouse_extra_info;
#[cfg(target_os = "windows")]
pub use crate::windows::{exit_grab, grab as _grab, is_grabbed};
#[cfg(target_os = "windows")]
pub use crate::windows::{set_event_popup, set_get_key_unicode};

/// Grabbing global events. In the callback, returning None ignores the event
/// and returning the event let's it pass. There is no modification of the event
/// possible here.
/// Caveat: On MacOS, you require the grab
/// loop needs to be the primary app (no fork before) and need to have accessibility
/// settings enabled.
/// On Linux, you need rw access to evdev devices in /etc/input/ (usually group membership in `input` group is enough)
///
/// ```no_run
/// use rdev::{grab, Event, EventType, Key};
///
/// fn callback(event: Event) -> Option<Event> {
///     println!("My callback {:?}", event);
///     match event.event_type{
///         EventType::KeyPress(Key::Tab) => None,
///         _ => Some(event),
///     }
/// }
/// fn main(){
///     // This will block.
///     if let Err(error) = grab(callback) {
///         println!("Error: {:?}", error)
///     }
/// }
/// ```
#[cfg(not(any(target_os = "android", target_os = "ios", target_os = "linux")))]
pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: Fn(Event) -> Option<Event> + 'static,
{
    _grab(callback)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn keyboard_only() -> bool {
    !std::env::var("KEYBOARD_ONLY")
        .unwrap_or_default()
        .is_empty()
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_keyboard_state() {
        // // S
        // let mut keyboard = Keyboard::new();
        // let char_s = keyboard
        //     .add(&EventType::KeyPress(Key::KeyS))
        //     .unwrap()
        //     .name
        //     .unwrap();
        // assert_eq!(
        //     char_s,
        //     "s".to_string(),
        //     "This test should pass only on Qwerty layout !"
        // );
        // let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        // assert_eq!(n, None);

        // // Shift + S
        // keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        // let char_s = keyboard
        //     .add(&EventType::KeyPress(Key::KeyS))
        //     .unwrap()
        //     .name
        //     .unwrap();
        // assert_eq!(char_s, "S".to_string());
        // let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        // assert_eq!(n, None);
        // keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // // Reset
        // keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        // let char_s = keyboard
        //     .add(&EventType::KeyPress(Key::KeyS))
        //     .unwrap()
        //     .name
        //     .unwrap();
        // assert_eq!(char_s, "s".to_string());
        // let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        // assert_eq!(n, None);
        // keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // UsIntl layout required
        // let n = keyboard.add(&EventType::KeyPress(Key::Quote));
        // assert_eq!(n, Some("".to_string()));
        // let m = keyboard.add(&EventType::KeyRelease(Key::Quote));
        // assert_eq!(m, None);
        // let e = keyboard.add(&EventType::KeyPress(Key::KeyE)).unwrap();
        // assert_eq!(e, "é".to_string());
        // keyboard.add(&EventType::KeyRelease(Key::KeyE));
    }
}
