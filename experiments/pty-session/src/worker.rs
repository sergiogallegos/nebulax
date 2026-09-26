//! Single worker owns PTY/engine/reaping. Readers see only owned latest frames.
use crate::{Session, Step};
use nebulax_terminal::{Size, Terminal, snapshot::Snapshot};
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::Command;
use std::sync::{
    Arc, Condvar, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Starting = 0,
    Running = 1,
    Exited = 2,
    Stopped = 3,
    Failed = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    pub phase: Phase,
    /// Normal exit code, or -1 for signal/no exit code.
    pub exit_code: i32,
    /// 0 none; 1 spawn, 2 transport, 3 resize, 4 snapshot limit, 5 panic.
    pub failure: u32,
    pub denied_effects: u64,
}
struct Published {
    status: Status,
    latest: Option<Arc<Snapshot>>,
    resize: Option<Size>,
}
struct Shared {
    stop: AtomicBool,
    state: Mutex<Published>,
    wake: Condvar,
}
impl Shared {
    fn lock(&self) -> MutexGuard<'_, Published> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}
pub struct Worker {
    shared: Arc<Shared>,
    thread: JoinHandle<()>,
}
impl Worker {
    /// Launch does not wait for PTY spawn; observe Starting/Running/Failed.
    pub fn start(command: Command, terminal: Terminal) -> io::Result<Self> {
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            wake: Condvar::new(),
            state: Mutex::new(Published {
                status: Status {
                    phase: Phase::Starting,
                    exit_code: -1,
                    failure: 0,
                    denied_effects: 0,
                },
                latest: None,
                resize: None,
            }),
        });
        let state = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("nebulax-session".into())
            .spawn(move || {
                // Session is created inside this boundary; unwinding drops/reaps it
                // on this worker before a terminal status can become observable.
                let result = catch_unwind(AssertUnwindSafe(|| run(command, terminal, &state)));
                let (phase, exit_code, failure) = result.unwrap_or((Phase::Failed, -1, 5));
                let mut published = state.lock();
                published.status.phase = phase;
                published.status.exit_code = exit_code;
                published.status.failure = failure;
            })?;
        Ok(Self { shared, thread })
    }
    pub fn status(&self) -> Status {
        self.shared.lock().status
    }
    pub fn latest(&self, after: u64) -> Option<Arc<Snapshot>> {
        self.shared
            .lock()
            .latest
            .as_ref()
            .filter(|f| f.generation() > after)
            .cloned()
    }
    /// Coalesces resize requests. Success acknowledges queuing, not OS completion.
    pub fn resize(&self, size: Size) -> bool {
        if !valid_geometry(size) || self.shared.stop.load(Ordering::Acquire) {
            return false;
        }
        let mut state = self.shared.lock();
        if !matches!(state.status.phase, Phase::Starting | Phase::Running) {
            return false;
        }
        state.resize = Some(size);
        self.shared.wake.notify_one();
        true
    }
    pub fn close(&self) {
        let _guard = self.shared.lock();
        self.shared.stop.store(true, Ordering::Release);
        self.shared.wake.notify_one();
    }
    /// Allows a native owner to release its slot only after cleanup has completed.
    pub fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.close();
    } // No joining or child waiting on caller.
}
pub fn valid_geometry(size: Size) -> bool {
    size.columns >= 2
        && size.lines > 0
        && size.columns <= u16::MAX as usize
        && size.lines <= u16::MAX as usize
        && size
            .columns
            .checked_mul(size.lines)
            .is_some_and(|n| n <= 65_536)
}
fn run(command: Command, terminal: Terminal, state: &Shared) -> (Phase, i32, u32) {
    if state.stop.load(Ordering::Acquire) {
        return (Phase::Stopped, -1, 0);
    }
    let mut session = match Session::spawn(command, terminal) {
        Ok(s) => s,
        Err(_) => return (Phase::Failed, -1, 1),
    };
    state.lock().status.phase = Phase::Running;
    let mut previous: Option<Arc<Snapshot>> = None;
    loop {
        if state.stop.load(Ordering::Acquire) {
            return (Phase::Stopped, -1, 0);
        }
        let resize = state.lock().resize.take();
        if let Some(size) = resize
            && session.resize(size).is_err()
        {
            return (Phase::Failed, -1, 3);
        }
        let mut denied = 0u64;
        let step = match session.tick(|_| {
            denied += 1;
            true
        }) {
            Ok(step) => step,
            Err(_) => return (Phase::Failed, -1, 2),
        };
        if denied > 0 {
            let mut s = state.lock();
            s.status.denied_effects = s.status.denied_effects.saturating_add(denied);
        }
        if session.take_changed() {
            let frame = match Snapshot::capture(session.pump().terminal(), previous.as_deref()) {
                Ok(frame) => Arc::new(frame),
                Err(_) => return (Phase::Failed, -1, 4),
            };
            state.lock().latest = Some(Arc::clone(&frame));
            previous = Some(frame);
        }
        if session.is_complete() {
            return (
                Phase::Exited,
                session.child_status().and_then(|s| s.code()).unwrap_or(-1),
                0,
            );
        }
        if step == Step::Yield {
            thread::yield_now();
            continue;
        }
        // Bounded prototype cadence, not final readiness scheduling. Control
        // predicate and condvar share the mutex to avoid lost resize/stop wakes.
        let guard = state.lock();
        if guard.resize.is_none() && !state.stop.load(Ordering::Acquire) {
            drop(
                state
                    .wake
                    .wait_timeout(guard, Duration::from_millis(2))
                    .unwrap_or_else(|e| e.into_inner()),
            );
        }
    }
}
