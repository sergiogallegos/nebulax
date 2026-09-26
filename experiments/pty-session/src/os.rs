//! Audited macOS boundary. Every descriptor is RAII-owned and close-on-exec.
use nebulax_terminal::Size;
use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

fn cvt(result: libc::c_int) -> io::Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn window(size: Size) -> io::Result<libc::winsize> {
    Ok(libc::winsize {
        ws_row: size
            .lines
            .try_into()
            .map_err(|_| io::ErrorKind::InvalidInput)?,
        ws_col: size
            .columns
            .try_into()
            .map_err(|_| io::ErrorKind::InvalidInput)?,
        ws_xpixel: 0,
        ws_ypixel: 0,
    })
}
pub(crate) fn validate_size(size: Size) -> io::Result<()> {
    window(size).map(|_| ())
}
pub(crate) fn resize(master: &File, size: Size) -> io::Result<()> {
    let window = window(size)?;
    // SAFETY: live owned PTY descriptor, platform ioctl and initialized winsize
    // pointer valid throughout the synchronous call. No pointer is retained.
    cvt(unsafe { libc::ioctl(master.as_raw_fd(), libc::TIOCSWINSZ, &window) })
}
pub(crate) fn spawn(mut command: Command, size: Size) -> io::Result<(File, Child)> {
    // Rust OpenOptions opens with O_CLOEXEC atomically, including master/slave.
    let master = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
        .open("/dev/ptmx")?;
    // SAFETY: grant/unlock operate only on the live owned master descriptor.
    cvt(unsafe { libc::grantpt(master.as_raw_fd()) })?;
    // SAFETY: same live master, no pointers or borrowed resources escape.
    cvt(unsafe { libc::unlockpt(master.as_raw_fd()) })?;
    let mut name = [0u8; 128];
    // SAFETY: Darwin TIOCPTYGNAME writes exactly its documented 128-byte buffer.
    // This avoids ptsname's shared static buffer and does not hand-write its ABI.
    cvt(unsafe {
        libc::ioctl(
            master.as_raw_fd(),
            libc::TIOCPTYGNAME.into(),
            name.as_mut_ptr(),
        )
    })?;
    let path = CStr::from_bytes_until_nul(&name).map_err(|_| io::ErrorKind::InvalidData)?;
    let slave = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open(std::ffi::OsStr::from_bytes(path.to_bytes()))?;
    resize(&master, size)?;
    command
        .stdin(Stdio::from(slave.try_clone()?))
        .stdout(Stdio::from(slave.try_clone()?))
        .stderr(Stdio::from(slave));
    // SAFETY: zero is valid storage for these C structs; sigemptyset/sigaction
    // fields establish their meaning before use. All preparation is before fork.
    let mut mask: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: points to initialized, correctly sized sigset_t storage.
    cvt(unsafe { libc::sigemptyset(&mut mask) })?;
    // SAFETY: Darwin sigaction has only integer/pointer fields, allowing zero.
    let mut default_action: libc::sigaction = unsafe { std::mem::zeroed() };
    default_action.sa_sigaction = libc::SIG_DFL;
    default_action.sa_mask = mask;
    // SAFETY: after fork this closure calls only platform setsid/ioctl and
    // signal syscalls, and constructs errno-only errors (no allocation, locks,
    // environment access, logging or Rust destructors with external state).
    // Command has already installed the slave on 0/1/2 before this hook.
    unsafe {
        command.pre_exec(move || {
            cvt(libc::setsid())?;
            cvt(libc::ioctl(0, libc::TIOCSCTTY.into(), 0))?;
            cvt(libc::sigprocmask(
                libc::SIG_SETMASK,
                &mask,
                std::ptr::null_mut(),
            ))?;
            for signal in [
                libc::SIGHUP,
                libc::SIGINT,
                libc::SIGQUIT,
                libc::SIGTERM,
                libc::SIGPIPE,
                libc::SIGCHLD,
                libc::SIGWINCH,
                libc::SIGTSTP,
                libc::SIGTTIN,
                libc::SIGTTOU,
            ] {
                cvt(libc::sigaction(
                    signal,
                    &default_action,
                    std::ptr::null_mut(),
                ))?;
            }
            Ok(())
        });
    }
    let child = command.spawn()?;
    // command owns parent-side slave Stdio handles: drop them before returning
    // so they cannot keep the slave open and hide EOF after the child exits.
    drop(command);
    Ok((master, child))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;
    use nebulax_terminal::{Limits, Terminal, WidthPolicy};

    #[test]
    fn dropping_session_kills_and_reaps_its_direct_child() {
        let terminal = Terminal::new(
            Size {
                columns: 8,
                lines: 3,
            },
            Limits::default(),
            WidthPolicy::default(),
        )
        .unwrap();
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let session = Session::spawn(command, terminal).unwrap();
        let pid = session.child_id() as libc::pid_t;
        drop(session);
        let mut status = 0;
        // SAFETY: pid belongs to this test's completed child; writable status is
        // valid. WNOHANG cannot wait on or signal an unrelated process.
        let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        assert_eq!(result, -1);
        assert_eq!(
            io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
    }
}
