#![cfg(target_os = "macos")]
use nebulax_pty_session::worker::{Phase, Worker};
use nebulax_terminal::{Limits, Size, Terminal, WidthPolicy};
use std::process::Command;
use std::time::{Duration, Instant};
fn start(program: &str, args: &[&str]) -> Worker {
    let mut c = Command::new(program);
    c.args(args);
    let t = Terminal::new(
        Size {
            columns: 8,
            lines: 3,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap();
    Worker::start(c, t).unwrap()
}
fn wait(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !predicate() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn worker_finishes_child_and_publishes_text_that_outlives_worker() {
    let w = start("/usr/bin/printf", &["owned 👩‍💻"]);
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    let frame = w.latest(0).unwrap();
    assert!(w.latest(frame.generation()).is_none());
    drop(w);
    assert_eq!(std::str::from_utf8(frame.text()).unwrap(), "owned 👩‍💻");
}
#[test]
fn close_is_a_request_and_cleanup_completes_on_worker() {
    let w = start("/bin/sleep", &["30"]);
    wait(|| w.latest(0).is_some());
    let frame = w.latest(0).unwrap();
    assert!(!w.resize(Size {
        columns: 1,
        lines: 3
    }));
    assert!(w.resize(Size {
        columns: 12,
        lines: 4
    }));
    wait(|| {
        w.latest(frame.generation())
            .is_some_and(|f| f.size().columns == 12)
    });
    w.close();
    w.close();
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Stopped);
    assert!(!w.resize(Size {
        columns: 12,
        lines: 4
    }));
    drop(w);
    assert_eq!(
        frame.size(),
        Size {
            columns: 8,
            lines: 3
        }
    );
}
#[test]
fn failed_launch_is_observable_and_slot_can_be_released() {
    let w = start("/nebulax-missing-worker-child", &[]);
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Failed);
    assert_eq!(w.status().failure, 1);
    assert!(w.latest(0).is_none());
}
#[test]
fn effects_are_denied_without_running_native_actions_or_stalling_replies() {
    let w = start(
        "python3",
        &[
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/child.py"),
            "effects",
        ],
    );
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.status().denied_effects, 600);
    assert_eq!(w.latest(0).unwrap().text(), b"DONE");
}

#[test]
fn worker_sends_native_text_and_keys_to_a_real_pty_peer() {
    use nebulax_pty_session::input::Input;
    use nebulax_terminal::input::Key;
    let w = start(
        "python3",
        &[
            "-c",
            "import os,tty;tty.setraw(0);os.write(1,b'READY');data=b''\nwhile len(data)<7:data+=os.read(0,7-len(data))\nassert data=='界'.encode()+b'\\r\\x1b[A';os.write(1,b'INPUT OK')",
        ],
    );
    wait(|| w.latest(0).is_some_and(|f| f.text().starts_with(b"READY")));
    w.input(Input::Text("界".into())).unwrap();
    w.input(Input::Key(Key::Enter)).unwrap();
    w.input(Input::Key(Key::Up)).unwrap();
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.latest(0).unwrap().text(), b"READYINPUT OK");
    assert!(w.input(Input::Text("late".into())).is_err());
}
