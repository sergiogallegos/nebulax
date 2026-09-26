//! Single-owner bounded PTY integration experiment, not a product runtime.
pub mod input;
mod pump;
pub use pump::{Pump, READ_CAPACITY, Step, Transport};

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod os;
#[cfg(target_os = "macos")]
mod session;
#[cfg(target_os = "macos")]
pub use session::Session;

#[cfg(target_os = "macos")]
pub mod worker;
