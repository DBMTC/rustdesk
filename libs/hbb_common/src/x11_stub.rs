//! X11 type stubs for a build that does not link libX11.
//!
//! Signatures match the `x11` crate's `xlib` module so callers can use both.

pub mod xlib {
    use crate::libc::{c_char, c_int, c_uchar, c_ulong, c_void};

    pub type Display = c_void;
    pub type Window = c_ulong;
    pub type Screen = c_void;

    pub unsafe fn XOpenDisplay(_: *const c_char) -> *mut Display {
        std::ptr::null_mut()
    }

    pub unsafe fn XCloseDisplay(_: *mut Display) {}

    pub unsafe fn XGetPointerMapping(_: *mut Display, _: *mut c_uchar, _: c_int) -> c_int {
        0
    }
}
