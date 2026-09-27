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

#[test]
fn pty_peer_negotiates_cursor_modes_origin_replies_and_region_scroll() {
    use nebulax_pty_session::input::Input;
    use nebulax_terminal::input::Key;
    let w = start(
        "python3",
        &[concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cursor_peer.py")],
    );
    wait(|| w.latest(0).is_some_and(|f| f.text() == b"TOPREADYbottom"));
    w.input(Input::Key(Key::Up)).unwrap();
    wait(|| w.latest(0).is_some_and(|f| f.text() == b"TOPNORMALSCROLL"));
    w.input(Input::Key(Key::Left)).unwrap();
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.latest(0).unwrap().text(), b"TOPPASSSCROLL");
}

#[test]
fn visibility_only_pty_output_publishes_a_new_frame_with_unchanged_rows() {
    use nebulax_pty_session::input::Input;
    let w = start(
        "python3",
        &[
            "-c",
            "import os,tty;tty.setraw(0);os.write(1,b'\\x1b[?7l123456789\\x1b[?25l');assert os.read(0,1)==b'v';os.write(1,b'\\x1b[?25h');os.read(0,1)",
        ],
    );
    wait(|| w.latest(0).is_some_and(|f| !f.cursor_visible()));
    let hidden = w.latest(0).unwrap();
    assert_eq!(hidden.text(), b"12345679");
    assert!(!hidden.cursor().wrap_pending);
    w.input(Input::Text("v".into())).unwrap();
    wait(|| {
        w.latest(hidden.generation())
            .is_some_and(|f| f.cursor_visible())
    });
    let shown = w.latest(hidden.generation()).unwrap();
    assert_eq!(shown.row_versions(), hidden.row_versions());
    assert_eq!(shown.cursor(), hidden.cursor());
    assert_eq!(shown.text(), hidden.text());
    w.close();
    wait(|| w.is_finished());
    drop(w);
    assert!(!hidden.cursor_visible() && shown.cursor_visible());
}

#[test]
fn application_startup_waits_for_identification_status_and_actual_mode_replies() {
    let w = start(
        "python3",
        &[
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/startup_peer.py"),
            env!("CARGO_PKG_VERSION"),
        ],
    );
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.status().denied_effects, 0);
    let frame = w.latest(0).unwrap();
    assert_eq!(frame.text(), b"START OK");
    assert!(frame.cursor_visible());
}

#[test]
fn soft_and_hard_reset_publish_frames_and_restore_native_key_encoding() {
    use nebulax_pty_session::input::Input;
    use nebulax_terminal::{input::Key, style::Style};
    let w = start(
        "python3",
        &[concat!(env!("CARGO_MANIFEST_DIR"), "/tests/reset_peer.py")],
    );
    wait(|| {
        w.latest(0)
            .is_some_and(|f| f.text() == b"KEEP" && !f.cursor_visible())
    });
    let before = w.latest(0).unwrap();
    w.input(Input::Text("s".into())).unwrap();
    wait(|| {
        w.latest(before.generation())
            .is_some_and(|f| f.cursor_visible())
    });
    let soft = w.latest(before.generation()).unwrap();
    assert_eq!(soft.row_versions(), before.row_versions());
    assert_eq!(soft.text(), before.text());
    assert_eq!(soft.cursor(), before.cursor());
    w.input(Input::Key(Key::Up)).unwrap();
    wait(|| {
        w.latest(soft.generation())
            .is_some_and(|f| f.text().is_empty() && !f.is_alternate() && f.cursor_visible())
    });
    let hard = w.latest(soft.generation()).unwrap();
    assert_eq!(hard.cursor(), nebulax_terminal::Cursor::default());
    assert_eq!(hard.styles(), [Style::default()]);
    w.input(Input::Text("r".into())).unwrap();
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.status().denied_effects, 0);
    assert_eq!(w.latest(0).unwrap().text(), b"RESET OK");
    drop(w);
    assert_eq!(before.text(), b"KEEP");
    assert_eq!(
        before.cell_style(&before.cells()[0]).unwrap().foreground,
        Style::indexed(1)
    );
    assert!(!before.cursor_visible() && soft.cursor_visible());
}

#[test]
fn explicit_paste_reaches_real_pty_with_negotiated_framing_and_reset_policy() {
    use nebulax_pty_session::input::{Input, InputError};
    let w = start(
        "python3",
        &[concat!(env!("CARGO_MANIFEST_DIR"), "/tests/paste_peer.py")],
    );
    for stage in 0..3 {
        wait(|| {
            w.latest(0)
                .is_some_and(|f| f.text() == format!("PASTE {stage}").as_bytes())
        });
        assert_eq!(
            w.input(Input::Paste("\x1b[201~".into())),
            Err(InputError::Invalid)
        );
        w.input(Input::Paste("界\ta\r\nb\nc\rd".into())).unwrap();
    }
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.latest(0).unwrap().text(), b"PASTE OK");
    assert_eq!(
        w.input(Input::Paste("late".into())),
        Err(InputError::Closed)
    );
}

#[test]
fn insert_mode_width_changes_and_resets_publish_correct_real_pty_frames() {
    use nebulax_pty_session::input::Input;
    let w = start(
        "python3",
        &[concat!(env!("CARGO_MANIFEST_DIR"), "/tests/insert_peer.py")],
    );
    wait(|| {
        w.latest(0)
            .is_some_and(|f| f.text() == "AB❤\u{fe0f}CDEFIRM ON".as_bytes())
    });
    let inserted = w.latest(0).unwrap();
    assert_eq!(inserted.cells()[2].width, 2);
    w.input(Input::Text("s".into())).unwrap();
    wait(|| {
        w.latest(inserted.generation())
            .is_some_and(|f| f.text() == b"ABZCDEFIRM OFF")
    });
    let replaced = w.latest(inserted.generation()).unwrap();
    assert_eq!(replaced.cells()[2].width, 1);
    assert_eq!(replaced.cells()[3].kind, 0);
    assert_ne!(inserted.row_versions()[0], replaced.row_versions()[0]);
    w.input(Input::Text("r".into())).unwrap();
    wait(|| w.is_finished());
    assert_eq!(w.status().phase, Phase::Exited);
    assert_eq!(w.status().exit_code, 0);
    assert_eq!(w.latest(0).unwrap().text(), b"IRM OK");
    drop(w);
    assert_eq!(inserted.text(), "AB❤\u{fe0f}CDEFIRM ON".as_bytes());
}
