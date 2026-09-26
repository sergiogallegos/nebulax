use nebulax_pty_session::input::{Input, InputError};
use nebulax_pty_session::worker::{Status, Worker, valid_geometry};
use nebulax_terminal::{Limits, Size, Terminal, WidthPolicy, snapshot::Snapshot};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

pub const OK: i32 = 0;
pub const INVALID: i32 = 1;
pub const LIMIT: i32 = 2;
pub const BUSY: i32 = 3;
pub const NO_FRAME: i32 = 4;
pub const FAILED: i32 = 5;
pub const PANIC: i32 = 6;
const SESSIONS: usize = 4;
const FRAMES: usize = 8;
struct SessionSlot {
    id: u64,
    worker: Worker,
}
struct FrameSlot {
    id: u64,
    owner: u64,
    frame: Arc<Snapshot>,
}
pub struct Registry {
    next: u64,
    sessions: [Option<SessionSlot>; SESSIONS],
    frames: [Option<FrameSlot>; FRAMES],
}
impl Default for Registry {
    fn default() -> Self {
        Self {
            next: 0,
            sessions: std::array::from_fn(|_| None),
            frames: std::array::from_fn(|_| None),
        }
    }
}
pub fn global() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}
impl Registry {
    fn id(&mut self) -> Result<u64, i32> {
        self.next = self.next.checked_add(1).ok_or(LIMIT)?;
        Ok(self.next)
    }
    fn worker(&self, id: u64) -> Result<&Worker, i32> {
        self.sessions
            .iter()
            .flatten()
            .find(|s| s.id == id)
            .map(|s| &s.worker)
            .ok_or(INVALID)
    }
    pub fn start(&mut self, command: Command, size: Size) -> Result<u64, i32> {
        if !valid_geometry(size) {
            return Err(INVALID);
        }
        let index = self
            .sessions
            .iter()
            .position(Option::is_none)
            .ok_or(LIMIT)?;
        let id = self.id()?;
        let terminal =
            Terminal::new(size, Limits::default(), WidthPolicy::default()).map_err(|_| INVALID)?;
        let worker = Worker::start(command, terminal).map_err(|_| FAILED)?;
        self.sessions[index] = Some(SessionSlot { id, worker });
        Ok(id)
    }
    pub fn status(&self, id: u64) -> Result<(Status, bool), i32> {
        let worker = self.worker(id)?;
        // Read completion first: a true value guarantees the later status read
        // observes final publication, rather than an earlier Running snapshot.
        let finished = worker.is_finished();
        Ok((worker.status(), finished))
    }
    pub fn resize(&self, id: u64, size: Size) -> Result<(), i32> {
        if !valid_geometry(size) {
            return Err(INVALID);
        }
        if self.worker(id)?.resize(size) {
            Ok(())
        } else {
            Err(BUSY)
        }
    }
    pub fn input(&self, id: u64, input: Input) -> Result<(), i32> {
        self.worker(id)?.input(input).map_err(|e| match e {
            InputError::Invalid => INVALID,
            InputError::Full => LIMIT,
            InputError::Closed => BUSY,
        })
    }
    pub fn close(&self, id: u64) -> Result<(), i32> {
        self.worker(id)?.close();
        Ok(())
    }
    pub fn release_session(&mut self, id: u64) -> Result<(), i32> {
        let index = self
            .sessions
            .iter()
            .position(|s| s.as_ref().is_some_and(|s| s.id == id))
            .ok_or(INVALID)?;
        if !self.sessions[index].as_ref().unwrap().worker.is_finished() {
            return Err(BUSY);
        }
        self.sessions[index] = None;
        Ok(())
    }
    pub fn acquire(&mut self, id: u64, after: u64) -> Result<u64, i32> {
        let worker = self.worker(id)?;
        if self
            .frames
            .iter()
            .flatten()
            .filter(|f| f.owner == id)
            .count()
            >= 2
        {
            return Err(LIMIT);
        }
        let index = self.frames.iter().position(Option::is_none).ok_or(LIMIT)?;
        let frame = worker.latest(after).ok_or(NO_FRAME)?;
        let frame_id = self.id()?;
        self.frames[index] = Some(FrameSlot {
            id: frame_id,
            owner: id,
            frame,
        });
        Ok(frame_id)
    }
    pub fn frame(&self, id: u64) -> Result<&Snapshot, i32> {
        self.frames
            .iter()
            .flatten()
            .find(|f| f.id == id)
            .map(|f| f.frame.as_ref())
            .ok_or(INVALID)
    }
    pub fn release_frame(&mut self, id: u64) -> Result<(), i32> {
        let index = self
            .frames
            .iter()
            .position(|f| f.as_ref().is_some_and(|f| f.id == id))
            .ok_or(INVALID)?;
        self.frames[index] = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    fn done(r: &Registry, id: u64) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !r.status(id).unwrap().1 {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn command() -> Command {
        let mut c = Command::new("/usr/bin/printf");
        c.arg("retained");
        c
    }
    #[test]
    fn session_and_frame_caps_survive_release_recreation_and_stale_handles() {
        let mut r = Registry::default();
        let size = Size {
            columns: 8,
            lines: 3,
        };
        let mut sessions = Vec::new();
        let mut frames = Vec::new();
        for _ in 0..4 {
            sessions.push(r.start(command(), size).unwrap());
        }
        assert_eq!(r.start(command(), size), Err(LIMIT));
        for id in &sessions {
            done(&r, *id);
            let first = r.acquire(*id, 0).unwrap();
            let generation = r.frame(first).unwrap().generation();
            assert_eq!(r.acquire(*id, generation), Err(NO_FRAME));
            frames.extend([first, r.acquire(*id, 0).unwrap()]);
            assert_eq!(r.acquire(*id, 0), Err(LIMIT));
            r.release_session(*id).unwrap();
            assert_eq!(r.close(*id), Err(INVALID));
        }
        let new = r.start(command(), size).unwrap();
        done(&r, new);
        assert!(!sessions.contains(&new));
        assert_eq!(r.acquire(new, 0), Err(LIMIT)); // Eight old leases still live.
        for id in frames {
            assert_eq!(r.frame(id).unwrap().text(), b"retained");
            r.release_frame(id).unwrap();
            assert_eq!(r.release_frame(id), Err(INVALID));
            assert!(r.frame(id).is_err());
        }
        let fresh = r.acquire(new, 0).unwrap();
        r.release_session(new).unwrap();
        assert_eq!(r.frame(fresh).unwrap().text(), b"retained");
        r.release_frame(fresh).unwrap();
    }
    #[test]
    fn closing_worker_keeps_its_slot_until_cleanup_and_handles_never_wrap() {
        let mut r = Registry::default();
        let mut c = Command::new("/bin/sleep");
        c.arg("30");
        let id = r
            .start(
                c,
                Size {
                    columns: 8,
                    lines: 3,
                },
            )
            .unwrap();
        assert_eq!(r.release_session(id), Err(BUSY));
        r.close(id).unwrap();
        done(&r, id);
        r.release_session(id).unwrap();
        r.next = u64::MAX;
        assert_eq!(
            r.start(
                command(),
                Size {
                    columns: 8,
                    lines: 3
                }
            ),
            Err(LIMIT)
        );
    }
}
