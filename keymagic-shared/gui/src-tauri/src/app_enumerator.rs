use anyhow::Result;
use crate::commands::AppInfo;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::get_running_apps;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::get_running_apps;

#[cfg(target_os = "linux")]
pub fn get_running_apps() -> Result<Vec<AppInfo>> {
    // Linux implementation placeholder
    // Could use /proc filesystem or D-Bus
    Ok(vec![])
}