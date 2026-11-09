//! No-op whiteboard for headless / Wayland-only Yocto builds.

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "t", content = "c")]
pub enum CustomEvent {
    Cursor(Cursor),
    Clear,
    Exit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "t")]
pub struct Cursor {
    pub x: f32,
    pub y: f32,
    pub argb: u32,
    pub btns: i32,
    pub text: String,
}

#[inline]
pub fn is_supported() -> bool {
    false
}

#[inline]
pub fn get_key_cursor(conn_id: i32) -> String {
    format!("{}-cursor", conn_id)
}

pub fn register_whiteboard(_k: String) {}

pub fn unregister_whiteboard(_k: String) {}

pub fn update_whiteboard(_k: String, _e: CustomEvent) {}

pub fn run() {}
