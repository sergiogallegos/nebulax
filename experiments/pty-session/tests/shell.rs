#![cfg(target_os = "macos")]
use nebulax_pty_session::{Session, input::Input};
use nebulax_terminal::{Cell, Limits, Size, Terminal, WidthPolicy, input::Key};
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn clean_dumb_shell_prompt_command_echo_and_exit_use_the_existing_protocol_subset() {
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
            .filter_map(|c| match c {
                Cell::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
                _ => None,
            })
            .collect();
        if sent == 0 && text.contains("nx> ") {
            s.queue_input(Input::Text("printf 'SHELL_OK\\n'; exit 0".into()))
                .unwrap();
            sent = 1;
        } else if sent == 1 && s.can_accept_input() {
            s.queue_input(Input::Key(Key::Enter)).unwrap();
            sent = 2;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(sent, 2);
    assert!(s.child_status().unwrap().success());
    let rows: Vec<String> = s
        .pump()
        .terminal()
        .screen()
        .iter()
        .map(|r| {
            r.cells()
                .iter()
                .filter_map(|c| match c {
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
