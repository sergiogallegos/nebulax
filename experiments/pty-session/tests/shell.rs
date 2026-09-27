#![cfg(target_os = "macos")]
use nebulax_pty_session::{Session, input::Input};
use nebulax_terminal::{CellView as Cell, Limits, Size, Terminal, WidthPolicy, input::Key};
use std::process::Command;
use std::time::{Duration, Instant};

fn run_clean_shell(script: &str) -> Session {
    // This is one controlled basic-shell scenario, not a compatibility claim.
    // No user startup scripts/history are loaded or written.
    let mut command = Command::new("/bin/bash");
    command
        .args(["--noprofile", "--norc", "-i"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TERM", "dumb")
        .env("HISTFILE", "/dev/null")
        .env("PS1", "nx> ");
    let terminal = Terminal::new(
        Size {
            columns: 80,
            lines: 24,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap();
    let mut s = Session::spawn(command, terminal).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut sent = 0;
    while !s.is_complete() {
        assert!(Instant::now() < deadline, "shell deadline");
        s.tick(|_| true).unwrap();
        let text: String = s
            .pump()
            .terminal()
            .screen()
            .iter()
            .flat_map(|r| r.cells())
            .filter_map(|c| match c.view() {
                Cell::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
                _ => None,
            })
            .collect();
        if sent == 0 && text.contains("nx> ") {
            s.queue_input(Input::Text(script.into())).unwrap();
            sent = 1;
        } else if sent == 1 && s.can_accept_input() {
            s.queue_input(Input::Key(Key::Enter)).unwrap();
            sent = 2;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(sent, 2);
    assert!(s.child_status().unwrap().success());
    assert!(
        !s.pump().diagnostics().unsupported,
        "clean shell required unsupported protocol"
    );
    s
}

#[test]
fn clean_dumb_shell_prompt_command_echo_and_exit_use_the_existing_protocol_subset() {
    let s = run_clean_shell("printf 'SHELL_OK\\n'; exit 0");
    let rows: Vec<String> = s
        .pump()
        .terminal()
        .screen()
        .iter()
        .map(|r| {
            r.cells()
                .iter()
                .filter_map(|c| match c.view() {
                    Cell::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
                    _ => None,
                })
                .collect()
        })
        .collect();
    assert!(rows.iter().any(|r| r == "SHELL_OK"));
    assert!(
        !s.pump().diagnostics().unsupported,
        "basic shell required unsupported protocol"
    );
}

#[test]
fn clean_shell_drives_tab_and_styled_wide_erasure_through_real_pty() {
    // Disable output processing so the PTY delivers the literal HT byte.
    // Emit UTF-8 with octal escapes: the clean shell has no locale configured.
    let s = run_clean_shell(
        "stty -echo -opost; printf '\\033[2J\\033[HLEFT\\tRIGHT\\033[2;1Habc\\347\\225\\214XYZ\\033[2;5H\\033[44m\\033[1K\\033[0m\\033[3;1HSHELL_TABS_ERASE_OK\\033[4;1H'; exit 0",
    );
    let t = s.pump().terminal();
    let frame = nebulax_terminal::snapshot::Snapshot::capture(t, None).unwrap();
    assert_eq!(frame.cell_text(&frame.cells()[8]), Some("R"));
    assert_eq!(frame.cell_text(&frame.cells()[80 + 5]), Some("X"));
    for cell in &frame.cells()[80..85] {
        assert_eq!(cell.kind, 0);
        assert_eq!(
            frame.cell_style(cell).unwrap().background,
            nebulax_terminal::style::Style::indexed(4)
        );
    }
    let text = std::str::from_utf8(frame.text()).unwrap();
    assert!(text.contains("SHELL_TABS_ERASE_OK"));
    assert!(t.invariants_hold());
}

#[test]
fn clean_shell_drives_character_and_region_line_edits() {
    let s = run_clean_shell(
        "stty -echo -opost; printf '\\033[2J\\033[HTOP\\033[2;1Habcd\\033[2;3H\\033[2@XY\\033[2;2H\\033[P\\033[7;1Habcd\\033[7;3H\\033[2@XY\\033[7;2H\\033[P\\033[3;1HMID\\033[4;1HLAST\\033[5;1HBOTTOM\\033[2;4r\\033[2;2H\\033[44m\\033[L\\033[3;1H\\033[M\\033[0m\\033[r\\033[6;1H'; exit 0",
    );
    let t = s.pump().terminal();
    let frame = nebulax_terminal::snapshot::Snapshot::capture(t, None).unwrap();
    let row_text = |row: usize| -> String {
        frame.cells()[row * 80..(row + 1) * 80]
            .iter()
            .filter_map(|c| frame.cell_text(c))
            .collect()
    };
    assert_eq!(row_text(6), "aXYcd");
    assert_eq!(row_text(0), "TOP");
    assert_eq!(row_text(1), "");
    assert_eq!(row_text(2), "MID");
    assert_eq!(row_text(3), "");
    assert_eq!(row_text(4), "BOTTOM");
    for row in [1, 3] {
        for cell in &frame.cells()[row * 80..(row + 1) * 80] {
            assert_eq!(cell.kind, 0);
            assert_eq!(
                frame.cell_style(cell).unwrap().background,
                nebulax_terminal::style::Style::indexed(4)
            );
        }
    }
    assert!(t.history().is_empty());
}
