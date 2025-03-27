//! Utility functions to register the wallpaper changer as a scheduled task.

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub(crate) use windows::{register_task, unregister_task};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub(crate) use linux::{register_task, unregister_task};
