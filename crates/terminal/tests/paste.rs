use nebulax_terminal::{Limits, OutputEvent, Size, Terminal, WidthPolicy, input::MAX_PASTE_BYTES};

fn terminal() -> Terminal {
    Terminal::new(
        Size {
            columns: 8,
            lines: 4,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn reply(t: &mut Terminal) -> Vec<u8> {
    let Some(OutputEvent::Reply(bytes)) = t.pop_output() else {
        panic!("missing reply")
    };
    bytes
}

#[test]
fn paste_mode_and_queries_work_at_every_stream_split() {
    let bytes = b"\x1b[?2004$p\x1b[?2004h\x1b[?2004$p\x1b[?2004l\x1b[?2004$p";
    for split in 0..=bytes.len() {
        let mut t = terminal();
        for part in [&bytes[..split], &bytes[split..]] {
            let out = t.feed(part);
            assert_eq!(out.consumed, part.len());
            assert!(!out.unsupported && !out.changed);
        }
        for state in [2, 1, 2] {
            assert_eq!(reply(&mut t), format!("\x1b[?2004;{state}$y").as_bytes());
        }
        assert!(!t.bracketed_paste() && t.invariants_hold());
        assert!(t.pop_output().is_none());
    }
}

#[test]
fn paste_normalizes_newlines_preserves_unicode_and_only_frames_explicit_paste() {
    use nebulax_terminal::input::Key;
    let mut t = terminal();
    let raw = "界\t👩‍💻\r\na\nb\rc\r\r\n\n";
    let expected = "界\t👩‍💻\ra\rb\rc\r\r\r";
    assert_eq!(t.encode_paste(raw).unwrap(), expected.as_bytes());
    t.feed(b"\x1b[?2004h");
    assert_eq!(
        t.encode_paste(raw).unwrap(),
        format!("\x1b[200~{expected}\x1b[201~").as_bytes()
    );
    assert_eq!(t.encode_key(Key::Enter).unwrap(), b"\r");
    assert_eq!(t.encode_key(Key::Up).unwrap(), b"\x1b[A");
}

#[test]
fn paste_limits_and_controls_are_rejected_without_partial_encoding() {
    let mut t = terminal();
    for enabled in [false, true] {
        if enabled {
            t.feed(b"\x1b[?2004h");
        }
        for c in (0..=0x9f)
            .filter_map(char::from_u32)
            .filter(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
        {
            assert!(
                t.encode_paste(&format!("before{c}after")).is_none(),
                "{c:?}"
            );
        }
        for invalid in [
            String::new(),
            "x".repeat(MAX_PASTE_BYTES + 1),
            "\x1b[201~command\n".into(),
            "\u{009b}201~".into(),
        ] {
            assert!(t.encode_paste(&invalid).is_none());
        }
        let maximum = "é".repeat(MAX_PASTE_BYTES / 2);
        assert_eq!(
            t.encode_paste(&maximum).unwrap().len(),
            MAX_PASTE_BYTES + if enabled { 12 } else { 0 }
        );
        assert!(t.encode_paste(&(maximum + "é")).is_none());
        assert!(t.pop_output().is_none());
    }
}

#[test]
fn paste_mode_is_global_and_resets_clear_it_without_rewriting_captured_bytes() {
    let mut t = terminal();
    t.feed(b"\x1b[?2004h\x1b7\x1b[?1049h\x1b[?2004l\x1b[?1049l\x1b8");
    assert!(!t.bracketed_paste());
    for reset in [b"\x1b[!p".as_slice(), b"\x1bc"] {
        t.feed(b"\x1b[?2004h\x1b[?1049h");
        t.resize(Size {
            columns: 6,
            lines: 3,
        })
        .unwrap();
        assert!(t.bracketed_paste());
        let encoded = t.encode_paste("saved").unwrap();
        t.feed(b"\x1b[?2004$p");
        t.feed(reset);
        assert!(!t.bracketed_paste());
        assert_eq!(reply(&mut t), b"\x1b[?2004;1$y");
        assert_eq!(encoded, b"\x1b[200~saved\x1b[201~");
        assert_eq!(t.encode_paste("next").unwrap(), b"next");
    }
}

#[test]
fn invalid_mode_lists_are_atomic_and_cannot_enable_paste() {
    let mut t = terminal();
    for bytes in [
        b"\x1b[?2004;2005h".as_slice(),
        b"\x1b[2004h",
        b"\x1b[?2004:1h",
        b"\x1b[?2004 h",
    ] {
        assert!(t.feed(bytes).unsupported);
        assert!(!t.bracketed_paste());
    }
    assert!(!t.feed(b"\x1b[?1;2004h").unsupported);
    assert!(t.bracketed_paste());
    assert!(t.feed(b"\x1b[?2004;2005l").unsupported);
    assert!(t.bracketed_paste());
}

#[test]
fn paste_mode_replies_retain_query_time_state_under_output_pressure() {
    let mut t = terminal();
    let mut bytes = Vec::new();
    for _ in 0..100 {
        bytes.extend_from_slice(b"\x1b[?2004h\x1b[?2004$p\x1b[?2004l\x1b[?2004$p");
    }
    let mut offset = 0;
    let mut replies = Vec::new();
    let mut blocked = false;
    while offset < bytes.len() {
        let out = t.feed(&bytes[offset..]);
        offset += out.consumed;
        blocked |= out.output_blocked;
        assert!(!out.unsupported);
        replies.push(reply(&mut t));
    }
    while let Some(OutputEvent::Reply(bytes)) = t.pop_output() {
        replies.push(bytes);
    }
    assert!(blocked);
    assert_eq!(replies.len(), 200);
    for (index, bytes) in replies.iter().enumerate() {
        assert_eq!(
            *bytes,
            format!("\x1b[?2004;{}$y", if index % 2 == 0 { 1 } else { 2 }).as_bytes()
        );
    }
    assert!(!t.bracketed_paste() && t.invariants_hold());
}
