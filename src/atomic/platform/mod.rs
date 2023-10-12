#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
pub mod platform;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
#[path = "macos.rs"]
pub mod platform;

#[cfg(windows)]
#[path = "windows.rs"]
pub mod platform;

#[cfg(target_os = "freebsd")]
#[path = "freebsd.rs"]
pub mod platform;
