//! Compile the actual owned declarations, independently of the C SDK oracle.
#[path = "../../src/os/bindings.rs"]
mod bindings;
use bindings::*;
use std::ffi::{c_int, c_ulong};
use std::mem::{align_of, offset_of, size_of};

fn layout<T>(name: &str) {
    println!("{name}.size={}", size_of::<T>());
    println!("{name}.align={}", align_of::<T>());
}
fn main() {
    let _: unsafe extern "C" fn(c_int, c_ulong, ...) -> c_int = ioctl;
    let _: unsafe extern "C" fn(c_int) -> c_int = grantpt;
    let _: unsafe extern "C" fn(c_int) -> c_int = unlockpt;
    let _: unsafe extern "C" fn() -> i32 = setsid;
    let _: unsafe extern "C" fn(*mut u32) -> c_int = sigemptyset;
    let _: unsafe extern "C" fn(c_int, *const u32, *mut u32) -> c_int = sigprocmask;
    let _: unsafe extern "C" fn(c_int, *const SignalAction, *mut SignalAction) -> c_int = sigaction;
    let _: unsafe extern "C" fn(i32, *mut c_int, c_int) -> i32 = waitpid;
    layout::<c_int>("int"); layout::<c_ulong>("ulong"); layout::<Pid>("pid"); layout::<SignalSet>("sigset");
    layout::<WindowSize>("winsize"); layout::<SignalAction>("sigaction");
    println!("winsize.rows={}", offset_of!(WindowSize, rows));
    println!("winsize.columns={}", offset_of!(WindowSize, columns));
    println!("winsize.xpixel={}", offset_of!(WindowSize, xpixel));
    println!("winsize.ypixel={}", offset_of!(WindowSize, ypixel));
    println!("sigaction.handler={}", offset_of!(SignalAction, handler));
    println!("sigaction.mask={}", offset_of!(SignalAction, mask));
    println!("sigaction.flags={}", offset_of!(SignalAction, flags));
    let mut mask: SignalSet = 123;
    // SAFETY: actual owned declaration, writable SDK-checked storage; no retention.
    assert_eq!(unsafe { sigemptyset(&mut mask) }, 0);
    println!("empty_mask={mask}");
    let action = SignalAction { handler: None, mask, flags: 0 };
    println!("default_handler_is_null={}", usize::from(action.handler.is_none()));
    for (name, value) in [("O_NOCTTY", O_NOCTTY as u64), ("O_NONBLOCK", O_NONBLOCK as u64),
        ("TIOCSWINSZ", TIOCSWINSZ), ("TIOCPTYGNAME", TIOCPTYGNAME), ("TIOCSCTTY", TIOCSCTTY),
        ("SIG_SETMASK", SIG_SETMASK as u64), ("WNOHANG", WNOHANG as u64), ("ECHILD", ECHILD as u64)] {
        println!("{name}={value}");
    }
    for (i, signal) in RESET_SIGNALS.iter().enumerate() { println!("signal.{i}={signal}"); }
}
