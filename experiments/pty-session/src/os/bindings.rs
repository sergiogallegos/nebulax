//! Owned subset of the macOS user-space C ABI, checked by scripts/pty-abi.
//! Derived from SDK declarations, not from the libc crate. No kernel syscalls.
use std::ffi::{c_int, c_ulong, c_ushort};

#[cfg(not(all(
    target_os = "macos",
    target_pointer_width = "64",
    any(target_arch = "aarch64", target_arch = "x86_64")
)))]
compile_error!("PTY bindings require a verified 64-bit macOS ABI; add SDK checks before extending");

pub(super) type Pid = i32;
pub(super) type SignalSet = u32;

#[repr(C)]
pub(super) struct WindowSize {
    pub rows: c_ushort,
    pub columns: c_ushort,
    pub xpixel: c_ushort,
    pub ypixel: c_ushort,
}

// Only used to install SIG_DFL, with flags = 0 and no old-action output.
// The SDK handler union occupies one function-pointer slot. None is its null
// SIG_DFL representation; no arbitrary handler address is read or called here.
#[repr(C)]
pub(super) struct SignalAction {
    pub handler: Option<unsafe extern "C" fn(c_int)>,
    pub mask: SignalSet,
    pub flags: c_int,
}

pub(super) const O_NOCTTY: c_int = 0x0002_0000;
pub(super) const O_NONBLOCK: c_int = 0x0000_0004;
pub(super) const TIOCSWINSZ: c_ulong = 0x8008_7467;
pub(super) const TIOCPTYGNAME: c_ulong = 0x4080_7453;
pub(super) const TIOCSCTTY: c_ulong = 0x2000_7461;
pub(super) const SIG_SETMASK: c_int = 3;
pub(super) const RESET_SIGNALS: [c_int; 10] = [1, 2, 3, 15, 13, 20, 28, 18, 21, 22];
#[cfg(test)]
pub(super) const WNOHANG: c_int = 1;
#[cfg(test)]
pub(super) const ECHILD: c_int = 10;

// std links libSystem on macOS. These declarations bind its public C wrappers,
// including libc's signal trampoline handling, never the kernel ABI.
unsafe extern "C" {
    pub(super) fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    pub(super) fn grantpt(fd: c_int) -> c_int;
    pub(super) fn unlockpt(fd: c_int) -> c_int;
    pub(super) fn setsid() -> Pid;
    pub(super) fn sigemptyset(set: *mut SignalSet) -> c_int;
    pub(super) fn sigprocmask(how: c_int, set: *const SignalSet, old: *mut SignalSet) -> c_int;
    pub(super) fn sigaction(
        signal: c_int,
        action: *const SignalAction,
        old: *mut SignalAction,
    ) -> c_int;
    #[cfg(test)]
    pub(super) fn waitpid(pid: Pid, status: *mut c_int, options: c_int) -> Pid;
}
