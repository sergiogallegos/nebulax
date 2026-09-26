//! Private macOS C ABI experiment. No Rust layouts or engine borrows escape.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod ffi;
#[cfg(target_os = "macos")]
mod registry;
