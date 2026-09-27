#![cfg(target_os = "macos")]
use nebulax_pty_session::{Session, Step};
use nebulax_terminal::{CellView as Cell, Limits, OutputEvent, Size, Terminal, WidthPolicy};
use std::io;
use std::process::Command;
use std::time::{Duration, Instant};

fn terminal() -> Terminal {
    Terminal::new(
        Size {
            columns: 8,
            lines: 3,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn spawn(mode: &str) -> Session {
    let mut command = Command::new("python3");
    command
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/child.py"))
        .arg(mode);
    Session::spawn(command, terminal()).unwrap()
}
fn text(session: &Session) -> String {
    session
        .pump()
        .terminal()
        .screen()
        .iter()
        .flat_map(|r| r.cells())
        .filter_map(|c| match c.view() {
            Cell::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
            _ => None,
        })
        .collect()
}
fn drive(session: &mut Session, mut tick: impl FnMut(&mut Session)) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !session.is_complete() {
        assert!(
            Instant::now() < deadline,
            "PTY deadline expired; screen {:?}",
            text(session)
        );
        tick(session);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn real_pty_roundtrip_controlling_terminal_resize_signal_and_eof() {
    let mut session = spawn("roundtrip");
    let mut bells = 0;
    let mut resized = false;
    drive(&mut session, |session| {
        let mut resize = false;
        session
            .tick(|e| {
                match e {
                    OutputEvent::Title { text, .. } => {
                        assert_eq!(text, "ready");
                        resize = true;
                    }
                    OutputEvent::Bell => bells += 1,
                    _ => panic!("replies must go to the PTY"),
                }
                true
            })
            .unwrap();
        if resize {
            session
                .resize(Size {
                    columns: 12,
                    lines: 4,
                })
                .unwrap();
            resized = true;
        }
    });
    assert!(resized);
    assert_eq!(bells, 1);
    assert_eq!(text(&session), "abRDONEERR�");
    assert!(session.child_status().unwrap().success());
    assert!(!session.pump().diagnostics().unsupported);
}
#[test]
fn real_effect_pressure_releases_without_event_or_reply_loss() {
    let mut session = spawn("effects");
    let deadline = Instant::now() + Duration::from_secs(5);
    while session.tick(|_| false).unwrap() != Step::EffectBlocked {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let frozen = session.pump().terminal().clone();
    for _ in 0..5 {
        assert_eq!(session.tick(|_| false).unwrap(), Step::EffectBlocked);
        assert_eq!(session.pump().terminal(), &frozen);
    }
    let mut count = 0;
    drive(&mut session, |s| {
        s.tick(|e| {
            assert_eq!(matches!(e, OutputEvent::Title { .. }), count % 2 == 0);
            count += 1;
            true
        })
        .unwrap();
    });
    assert_eq!(count, 600);
    assert_eq!(text(&session), "DONE");
    assert!(session.child_status().unwrap().success());
}
#[test]
fn eof_and_nonzero_exit_are_independent_and_tail_output_is_drained() {
    let mut session = spawn("eof_first");
    let mut eof_before_exit = false;
    drive(&mut session, |s| {
        let step = s.tick(|_| true).unwrap();
        eof_before_exit |= step == Step::Finished && s.child_status().is_none();
    });
    assert!(eof_before_exit);
    assert_eq!(session.child_status().unwrap().code(), Some(7));
    assert_eq!(text(&session), "EOF");
    let mut session = spawn("nonzero");
    drive(&mut session, |s| {
        s.tick(|_| true).unwrap();
    });
    assert_eq!(session.child_status().unwrap().code(), Some(9));
    assert_eq!(text(&session), "LAST");
}
#[test]
fn invalid_resize_is_atomic_and_shutdown_reaps_is_idempotent() {
    let mut session = spawn("idle");
    let before = session.pump().terminal().clone();
    for size in [
        Size {
            columns: 1,
            lines: 2,
        },
        Size {
            columns: 65536,
            lines: 1,
        },
    ] {
        assert_eq!(
            session.resize(size).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(session.pump().terminal(), &before);
    }
    let status = session.shutdown().unwrap();
    assert!(!status.success());
    assert_eq!(session.shutdown().unwrap(), status);
    assert_eq!(
        session.tick(|_| true).unwrap_err().kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(
        session
            .resize(Size {
                columns: 12,
                lines: 4
            })
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(session.pump().terminal(), &before);
    assert!(!session.is_complete()); // Cancellation is not a clean drain.
}
#[test]
fn failed_spawn_releases_handles_and_repeated_sessions_finish() {
    let command = Command::new("/nebulax-nonexistent-synthetic-child");
    assert_eq!(
        Session::spawn(command, terminal()).err().unwrap().kind(),
        io::ErrorKind::NotFound
    );
    for _ in 0..8 {
        let mut session = spawn("nonzero");
        drive(&mut session, |s| {
            s.tick(|_| true).unwrap();
        });
        assert_eq!(session.child_status().unwrap().code(), Some(9));
    }
}
