//! Audited macOS boundary. Every descriptor is RAII-owned and close-on-exec.
mod bindings;
use bindings as abi;
use nebulax_terminal::Size;
use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

fn cvt(result: std::ffi::c_int) -> io::Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn window(size: Size) -> io::Result<abi::WindowSize> {
    Ok(abi::WindowSize {
        rows: size
            .lines
            .try_into()
            .map_err(|_| io::ErrorKind::InvalidInput)?,
        columns: size
            .columns
            .try_into()
            .map_err(|_| io::ErrorKind::InvalidInput)?,
        xpixel: 0,
        ypixel: 0,
    })
}
pub(crate) fn validate_size(size: Size) -> io::Result<()> {
    window(size).map(|_| ())
}
pub(crate) fn resize(master: &File, size: Size) -> io::Result<()> {
    let window = window(size)?;
    // SAFETY: live owned PTY descriptor, platform ioctl and initialized winsize
    // pointer valid throughout the synchronous call. No pointer is retained.
    cvt(unsafe { abi::ioctl(master.as_raw_fd(), abi::TIOCSWINSZ, &window) })
}
pub(crate) fn spawn(mut command: Command, size: Size) -> io::Result<(File, Child)> {
    // Rust OpenOptions opens with O_CLOEXEC atomically, including master/slave.
    let master = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(abi::O_NOCTTY | abi::O_NONBLOCK)
        .open("/dev/ptmx")?;
    // SAFETY: grant/unlock operate only on the live owned master descriptor.
    cvt(unsafe { abi::grantpt(master.as_raw_fd()) })?;
    // SAFETY: same live master, no pointers or borrowed resources escape.
    cvt(unsafe { abi::unlockpt(master.as_raw_fd()) })?;
    let mut name = [0u8; 128];
    // SAFETY: Darwin TIOCPTYGNAME writes exactly its documented 128-byte buffer.
    // The request and buffer extent are checked against the installed SDK.
    cvt(unsafe { abi::ioctl(master.as_raw_fd(), abi::TIOCPTYGNAME, name.as_mut_ptr()) })?;
    let path = CStr::from_bytes_until_nul(&name).map_err(|_| io::ErrorKind::InvalidData)?;
    let slave = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(abi::O_NOCTTY)
        .open(std::ffi::OsStr::from_bytes(path.to_bytes()))?;
    resize(&master, size)?;
    command
        .stdin(Stdio::from(slave.try_clone()?))
        .stdout(Stdio::from(slave.try_clone()?))
        .stderr(Stdio::from(slave));
    let mut mask: abi::SignalSet = 0;
    // SAFETY: points to initialized storage with the SDK-verified sigset_t ABI.
    cvt(unsafe { abi::sigemptyset(&mut mask) })?;
    let default_action = abi::SignalAction {
        handler: None,
        mask,
        flags: 0,
    };
    // SAFETY: after fork this closure calls only platform setsid/ioctl and
    // signal syscalls, and constructs errno-only errors (no allocation, locks,
    // environment access, logging or Rust destructors with external state).
    // Command has already installed the slave on 0/1/2 before this hook.
    unsafe {
        command.pre_exec(move || {
            cvt(abi::setsid())?;
            cvt(abi::ioctl(
                0,
                abi::TIOCSCTTY,
                std::ptr::null_mut::<std::ffi::c_void>(),
            ))?;
            cvt(abi::sigprocmask(
                abi::SIG_SETMASK,
                &mask,
                std::ptr::null_mut(),
            ))?;
            for signal in abi::RESET_SIGNALS {
                cvt(abi::sigaction(
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
    fn inherited_signals_are_reset_in_pty_child() {
        use std::time::{Duration, Instant};
        const CHILD: &str = "NEBULAX_ABI_TEST_PEER";
        if let Some(peer) = std::env::var_os(CHILD) {
            // This branch runs only in an isolated test process whose launcher
            // altered its signals. Never change the main test runner's handlers.
            assert!(
                Command::new(&peer)
                    .arg("--inherited")
                    .status()
                    .unwrap()
                    .success(),
                "signal preconditions were not inherited by the isolated runner"
            );
            let mut command = Command::new(peer);
            command.arg("--child");
            let terminal = Terminal::new(
                Size {
                    columns: 8,
                    lines: 3,
                },
                Limits::default(),
                WidthPolicy::default(),
            )
            .unwrap();
            let mut session = Session::spawn(command, terminal).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !session.is_complete() {
                assert!(Instant::now() < deadline);
                session.tick(|_| true).unwrap();
                std::thread::sleep(Duration::from_millis(1));
            }
            assert!(
                session.child_status().unwrap().success(),
                "SDK peer rejected inherited signals"
            );
            return;
        }
        let directory =
            std::env::temp_dir().join(format!("nebulax-signal-abi-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        let peer = directory.join("sdk-peer");
        let compiled = Command::new("xcrun")
            .args(["clang", "-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/abi/sdk.c"))
            .arg("-o")
            .arg(&peer)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let result = Command::new("python3").arg("-c").arg(
            "import os,signal,sys;\nfor s in (signal.SIGHUP,signal.SIGWINCH,signal.SIGTTOU):signal.signal(s,signal.SIG_IGN)\nsignal.pthread_sigmask(signal.SIG_BLOCK,{signal.SIGHUP,signal.SIGTERM,signal.SIGWINCH})\nos.execv(sys.argv[1],[sys.argv[1],'--exact','os::tests::inherited_signals_are_reset_in_pty_child','--nocapture'])"
        ).arg(std::env::current_exe().unwrap()).env(CHILD, &peer).output().unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains("test result: ok. 1 passed"));
    }

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
        let pid = session.child_id() as abi::Pid;
        drop(session);
        let mut status = 0;
        // SAFETY: pid belongs to this test's completed child; writable status is
        // valid. WNOHANG cannot wait on or signal an unrelated process.
        let result = unsafe { abi::waitpid(pid, &mut status, abi::WNOHANG) };
        assert_eq!(result, -1);
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(abi::ECHILD));
    }
}
