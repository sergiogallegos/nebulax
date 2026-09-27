use nebulax_terminal::{Limits, OutputEvent, Size, Terminal, WidthPolicy};

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
fn drain(t: &mut Terminal) -> Vec<Vec<u8>> {
    let mut replies = Vec::new();
    while let Some(event) = t.pop_output() {
        let OutputEvent::Reply(bytes) = event else {
            panic!("unexpected effect")
        };
        replies.push(bytes);
    }
    replies
}
fn version() -> Vec<u8> {
    concat!(
        "\x1bP>|Nebulax ",
        env!("CARGO_PKG_VERSION"),
        " (experimental)\x1b\\"
    )
    .as_bytes()
    .to_vec()
}
fn stream(parts: &[&[u8]]) -> (Terminal, Vec<Vec<u8>>) {
    let mut t = terminal();
    let mut replies = Vec::new();
    for part in parts {
        let out = t.feed(part);
        assert_eq!(out.consumed, part.len());
        assert!(!out.unsupported && !out.parser_limit && !out.output_blocked);
        replies.extend(drain(&mut t));
        assert!(t.invariants_hold());
    }
    (t, replies)
}

#[test]
fn explicit_identification_profile_accepts_only_default_or_zero_requests() {
    let (_, replies) = stream(&[b"\x1b[c\x1b[0c\x1bZ\x1b[>c\x1b[>0c\x1b[>q\x1b[>0q"]);
    assert_eq!(
        replies,
        [
            b"\x1b[?1;0c".to_vec(),
            b"\x1b[?1;0c".to_vec(),
            b"\x1b[?1;0c".to_vec(),
            b"\x1b[>0;1;0c".to_vec(),
            b"\x1b[>0;1;0c".to_vec(),
            version(),
            version()
        ]
    );
}

#[test]
fn mixed_queries_are_identical_at_every_byte_split_and_byte_at_a_time() {
    let bytes = b"ab\x1b[c\x1b[>0c\x1b[>q\x1b[5n\x1b[6n\x1b[?7$p\x1b[?25l\x1b[?25$p\x1b[4$pZ";
    let expected = stream(&[bytes]);
    for split in 0..=bytes.len() {
        assert_eq!(
            stream(&[&bytes[..split], &bytes[split..]]),
            expected,
            "split {split}"
        );
    }
    assert_eq!(stream(&bytes.chunks(1).collect::<Vec<_>>()), expected);
    assert_eq!(
        expected.1,
        [
            b"\x1b[?1;0c".to_vec(),
            b"\x1b[>0;1;0c".to_vec(),
            version(),
            b"\x1b[0n".to_vec(),
            b"\x1b[1;3R".to_vec(),
            b"\x1b[?7;1$y".to_vec(),
            b"\x1b[?25;2$y".to_vec(),
            b"\x1b[4;2$y".to_vec()
        ]
    );
}

#[test]
fn mode_reports_distinguish_implemented_reset_from_unrecognized_modes() {
    let mut t = terminal();
    for (number, initial) in [(1, 2), (6, 2), (7, 1), (25, 1), (1049, 2)] {
        for (command, state) in [("", initial), ("h", 1), ("l", 2)] {
            if !command.is_empty() {
                t.feed(format!("\x1b[?{number}{command}").as_bytes());
            }
            t.feed(format!("\x1b[?{number}$p").as_bytes());
            assert_eq!(
                drain(&mut t),
                [format!("\x1b[?{number};{state}$y").into_bytes()]
            );
        }
    }
    t.feed(b"\x1b[?2005$p\x1b[?0$p\x1b[?65535$p\x1b[20$p\x1b[7$p");
    assert_eq!(
        drain(&mut t),
        [
            b"\x1b[?2005;0$y".to_vec(),
            b"\x1b[?0;0$y".to_vec(),
            b"\x1b[?65535;0$y".to_vec(),
            b"\x1b[20;0$y".to_vec(),
            b"\x1b[7;0$y".to_vec()
        ]
    );
    assert!(t.feed(b"\x1b[?7;2005h").unsupported);
    t.feed(b"\x1b[?7$p");
    assert_eq!(drain(&mut t), [b"\x1b[?7;2$y".to_vec()]);
}

#[test]
fn replies_capture_query_time_screen_saved_modes_and_origin_coordinates() {
    let mut t = terminal();
    t.feed(b"\x1b[2;4r\x1b[?6h\x1b[?7l\x1b[2;3H\x1b7\x1b[6n\x1b[?6$p\x1b[?7$p");
    t.feed(b"\x1b[?6l\x1b[?7h\x1b8\x1b[?7$p\x1b[?1049h\x1b[?6$p\x1b[?7$p\x1b[?1049$p");
    t.resize(Size {
        columns: 10,
        lines: 5,
    })
    .unwrap();
    t.feed(b"\x1b[?1049l\x1b[?7$p\x1b[?1049$p\x1b[5n");
    assert_eq!(
        drain(&mut t),
        [
            b"\x1b[2;3R".to_vec(),
            b"\x1b[?6;1$y".to_vec(),
            b"\x1b[?7;2$y".to_vec(),
            b"\x1b[?7;2$y".to_vec(),
            b"\x1b[?6;2$y".to_vec(),
            b"\x1b[?7;1$y".to_vec(),
            b"\x1b[?1049;1$y".to_vec(),
            b"\x1b[?7;2$y".to_vec(),
            b"\x1b[?1049;2$y".to_vec(),
            b"\x1b[0n".to_vec()
        ]
    );
    assert!(t.invariants_hold());
}

#[test]
fn queries_preserve_display_rendition_cursor_and_pending_wrap() {
    let mut t = terminal();
    t.feed(b"\x1b[31;44mabcdefgh\0"); // Explicitly close grapheme attachment before comparison.
    let before = t.clone();
    let out = t.feed(b"\x1b[c\x1b[>c\x1b[>q\x1b[5n\x1b[6n\x1b[?7$p");
    assert!(!out.changed && !out.unsupported);
    assert_eq!(drain(&mut t).len(), 6);
    assert_eq!(t, before);
}

#[test]
fn malformed_cancelled_and_oversized_queries_never_reply() {
    for bytes in [
        b"\x1b[1c".as_slice(),
        b"\x1b[>1c",
        b"\x1b[>1q",
        b"\x1b[?6n",
        b"\x1b[0;0c",
        b"\x1b[>0:q",
        b"\x1b[?7;25$p",
        b"\x1b[?7:0$p",
        b"\x1b[?$p",
        b"\x1b[$p",
        b"\x1b[>7$p",
        b"\x1b[7 $p",
        b"\x1b(Z",
    ] {
        for split in 0..=bytes.len() {
            let mut t = terminal();
            let first = t.feed(&bytes[..split]);
            let second = t.feed(&bytes[split..]);
            assert!(first.unsupported || second.unsupported, "{bytes:?}");
            assert!(drain(&mut t).is_empty());
            assert!(t.invariants_hold());
        }
    }
    let mut t = terminal();
    assert!(t.feed(b"\x1b[?65536$p").parser_limit);
    t.feed(b"\x1b[>0\x18q\x1b[?7\x1a$p");
    assert!(drain(&mut t).is_empty());
}

#[test]
fn echoed_replies_do_not_create_response_loops() {
    let (mut t, replies) = stream(&[b"\x1b[c\x1b[>c\x1b[>q\x1b[5n\x1b[6n\x1b[?7$p\x1b[4$p"]);
    for reply in replies {
        t.feed(&reply);
    }
    assert!(drain(&mut t).is_empty());
    assert!(t.invariants_hold());
}

#[test]
fn sustained_query_pressure_resumes_exactly_and_preserves_order() {
    use nebulax_terminal::output::{
        MAX_EVENT_PAYLOAD_BYTES, MAX_QUEUED_EVENTS, MAX_QUEUED_PAYLOAD_BYTES,
    };
    let unit = b"\x1b[c\x1b[>q\x1b[?7l\x1b[?7$p\x1b[?7h\x1b[?7$p";
    let mut input = unit.repeat(100);
    input.push(b'Z');
    let mut expected = Vec::new();
    for _ in 0..100 {
        expected.extend([
            b"\x1b[?1;0c".to_vec(),
            version(),
            b"\x1b[?7;2$y".to_vec(),
            b"\x1b[?7;1$y".to_vec(),
        ]);
    }
    let mut t = terminal();
    let mut offset = 0;
    let mut actual = Vec::new();
    let mut blocked = 0;
    while offset < input.len() {
        let out = t.feed(&input[offset..]);
        assert!(out.consumed > 0);
        offset += out.consumed;
        assert!(t.pending_output_len() <= MAX_QUEUED_EVENTS + 1);
        assert!(t.pending_output_bytes() <= MAX_QUEUED_PAYLOAD_BYTES + MAX_EVENT_PAYLOAD_BYTES);
        if out.output_blocked {
            blocked += 1;
            if actual.is_empty() {
                assert_eq!(offset, unit.len() * 16 + 3);
            }
            let before = t.clone();
            assert_eq!(t.feed(&input[offset..]).consumed, 0);
            assert_eq!(t, before);
        }
        actual.extend(drain(&mut t));
        assert!(t.invariants_hold());
    }
    assert!(blocked > 1);
    assert_eq!(actual, expected);
    assert_eq!(t.cursor().column, 1); // Trailing Z was delivered once.
}
