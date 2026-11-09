//! rdev facade.
//!
//! The `x11` feature re-exports upstream rdev. Without it, the crate is the
//! Wayland build of the same API and does not link libX11.

#[cfg(feature = "x11")]
pub use rdev_upstream::*;

#[cfg(not(feature = "x11"))]
include!("stub_body.rs");
