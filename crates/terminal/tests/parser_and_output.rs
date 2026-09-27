use nebulax_terminal::output::{
    MAX_EVENT_PAYLOAD_BYTES, MAX_QUEUED_EVENTS, MAX_QUEUED_PAYLOAD_BYTES,
};
use nebulax_terminal::parser::{
    Event, Header, MAX_HEADER_BYTES, MAX_OSC_BYTES, MAX_PARAMETERS, Parser, StringKind,
};
use nebulax_terminal::{
    CellView as Cell, FeedOutcome, Limits, OutputEvent, Size, Terminal, TitleTarget, WidthPolicy,
};

fn terminal() -> Terminal {
    Terminal::new(
        Size {
            columns: 16,
            lines: 3,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
fn events(input: &[u8]) -> Vec<Event> {
    let mut parser = Parser::default();
    input.iter().filter_map(|b| parser.push(*b)).collect()
}
fn csi(input: &[u8]) -> Header {
    let parsed = events(input);
    match parsed.last().unwrap() {
        Event::Csi { header, .. } => *header,
        e => panic!("expected CSI: {e:?}"),
    }
}
fn drain(t: &mut Terminal) -> Vec<OutputEvent> {
    let mut result = Vec::new();
    while let Some(e) = t.pop_output() {
        result.push(e);
    }
    result
}
fn text(t: &Terminal) -> String {
    t.screen()[0]
        .cells()
        .iter()
        .filter_map(|c| match c.view() {
            Cell::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
            _ => None,
        })
        .collect()
}
fn stream(parts: &[&[u8]]) -> (Terminal, Vec<OutputEvent>, FeedOutcome) {
    let mut t = terminal();
    let mut output = Vec::new();
    let mut all = FeedOutcome::default();
    for part in parts {
        let mut offset = 0;
        while offset < part.len() {
            let progress = t.feed(&part[offset..]);
            assert!(progress.consumed > 0 || progress.output_blocked);
            offset += progress.consumed;
            all.merge(progress);
            output.extend(drain(&mut t));
            assert!(t.invariants_hold());
        }
    }
    // Blocking is a transport condition, not a delivery-invariant diagnostic.
    all.output_blocked = false;
    (t, output, all)
}
fn partitions(input: &[u8]) -> (Terminal, Vec<OutputEvent>, FeedOutcome) {
    let expected = stream(&[input]);
    for split in 0..=input.len() {
        assert_eq!(
            stream(&[&input[..split], &input[split..]]),
            expected,
            "split {split}"
        );
    }
    for size in [1, 2, 3, 7] {
        assert_eq!(stream(&input.chunks(size).collect::<Vec<_>>()), expected);
    }
    expected
}

#[test]
fn syntax_preserves_parameters_subparameters_defaults_and_intermediates() {
    let h = csi(b"\x1b[38:2::255:0:1;4:3m");
    let pairs: Vec<_> = h
        .parameters()
        .iter()
        .map(|p| (p.value, p.subparameter))
        .collect();
    assert_eq!(
        pairs,
        [
            (Some(38), false),
            (Some(2), true),
            (None, true),
            (Some(255), true),
            (Some(0), true),
            (Some(1), true),
            (Some(4), false),
            (Some(3), true)
        ]
    );
    assert!(csi(b"\x1b[m").parameters().is_empty());
    assert_eq!(
        csi(b"\x1b[;0:m")
            .parameters()
            .iter()
            .map(|p| p.value)
            .collect::<Vec<_>>(),
        [None, Some(0), None]
    );
    let h = csi(b"\x1b[?1049$p");
    assert_eq!(h.prefix(), Some(b'?'));
    assert_eq!(h.intermediates(), b"$");
    assert_eq!(h.parameters()[0].value, Some(1049));
    assert!(
        matches!(events(b"\x1b(0").last(), Some(Event::Esc { header, final_byte: b'0' }) if header.intermediates()==b"(")
    );
}

#[test]
fn syntax_limits_accept_exact_bounds_and_never_dispatch_truncated_headers() {
    let legal = format!("\x1b[{}m", vec!["1"; MAX_PARAMETERS].join(";"));
    assert_eq!(csi(legal.as_bytes()).parameters().len(), MAX_PARAMETERS);
    assert_eq!(csi(b"\x1b[65535D").parameters()[0].value, Some(65535));
    assert_eq!(
        csi(format!("\x1b[{}D", "0".repeat(MAX_HEADER_BYTES)).as_bytes()).parameters()[0].value,
        Some(0)
    );
    for input in [
        format!("\x1b[{}mZ", vec!["1"; MAX_PARAMETERS + 1].join(";")),
        "\x1b[65536DZ".to_owned(),
        format!("\x1b[{}DZ", "0".repeat(MAX_HEADER_BYTES + 1)),
        "\x1b[   qZ".to_owned(),
        "\x1b   AZ".to_owned(),
    ] {
        let parsed = events(input.as_bytes());
        assert_eq!(parsed.iter().filter(|e| **e == Event::Limit).count(), 1);
        assert!(
            !parsed
                .iter()
                .any(|e| matches!(e, Event::Csi { .. } | Event::Esc { .. }))
        );
        assert_eq!(parsed.last(), Some(&Event::Print(b'Z')));
    }
}

#[test]
fn control_execution_cancellation_and_escape_restart_preserve_syntax_order() {
    let parsed = events(b"\x1b[12\x07;3H");
    assert_eq!(parsed[1], Event::Execute(7));
    let Event::Csi { header, final_byte } = parsed[2] else {
        panic!()
    };
    assert_eq!(final_byte, b'H');
    assert_eq!(
        header
            .parameters()
            .iter()
            .map(|p| p.value)
            .collect::<Vec<_>>(),
        [Some(12), Some(3)]
    );
    for cancel in [0x18, 0x1a] {
        let mut input = b"\x1b[99".to_vec();
        input.extend([cancel, b'Z']);
        assert!(
            !events(&input)
                .iter()
                .any(|e| matches!(e, Event::Csi { .. }))
        );
    }
    let parsed = events(b"\x1b[999\x1b[6n");
    assert_eq!(
        parsed
            .iter()
            .filter(|e| matches!(e, Event::Csi { .. }))
            .count(),
        1
    );
    assert_eq!(csi(b"\x1b[999\x1b[6n").parameters()[0].value, Some(6));
}

#[test]
fn malformed_syntax_recovers_without_parameter_reinterpretation() {
    for bad in [
        b"\x1b[??1049h".as_slice(),
        b"\x1b[1?1049h",
        b"\x1b[1 2D",
        b"\x1b[\xffD",
    ] {
        let parsed = events(bad);
        assert!(parsed.contains(&Event::Invalid));
        assert!(!parsed.iter().any(|e| matches!(e, Event::Csi { .. })));
    }
    let (t, output, out) = partitions(b"abc\x1b[1:2D\x1b[?1049;999hZ");
    assert!(out.unsupported && !t.is_alternate());
    assert!(output.is_empty());
    assert_eq!(text(&t), "abcZ");
}

#[test]
fn bounded_osc_dispatches_only_complete_payloads_and_discards_other_strings() {
    assert_eq!(
        events(b"\x1b]2;title\x07").last(),
        Some(&Event::Osc(b"2;title".to_vec()))
    );
    assert_eq!(
        events(b"\x1b]2;title\x1b\\").last(),
        Some(&Event::Osc(b"2;title".to_vec()))
    );
    let legal = format!("\x1b]{}\x07", "x".repeat(MAX_OSC_BYTES));
    assert!(
        matches!(events(legal.as_bytes()).last(),Some(Event::Osc(b)) if b.len()==MAX_OSC_BYTES)
    );
    let long = format!("\x1b]2;{}\x1b\\Z", "x".repeat(100_000));
    let parsed = events(long.as_bytes());
    assert_eq!(parsed.iter().filter(|e| **e == Event::Limit).count(), 1);
    assert!(!parsed.iter().any(|e| matches!(e, Event::Osc(_))));
    assert_eq!(parsed.last(), Some(&Event::Print(b'Z')));
    for (introducer, kind) in [
        (b'P', StringKind::Dcs),
        (b'_', StringKind::Apc),
        (b'^', StringKind::Pm),
        (b'X', StringKind::Sos),
    ] {
        let mut bytes = vec![0x1b, introducer];
        bytes.extend_from_slice(b"\x07\x1b[6n\x1b\\Z");
        assert_eq!(
            events(&bytes),
            [
                Event::EscapeBoundary,
                Event::IgnoredString(kind),
                Event::Print(b'Z')
            ]
        );
    }
}

#[test]
fn replies_bell_and_titles_are_typed_ordered_and_partition_invariant() {
    let (t, output, out) =
        partitions("ab\u{1b}[5n\u{7}\u{1b}]2;héllo\u{1b}\\\u{1b}[6nZ".as_bytes());
    assert_eq!(
        output,
        [
            OutputEvent::Reply(b"\x1b[0n".to_vec()),
            OutputEvent::Bell,
            OutputEvent::Title {
                target: TitleTarget::Window,
                text: "héllo".to_owned()
            },
            OutputEvent::Reply(b"\x1b[1;3R".to_vec())
        ]
    );
    assert!(!out.unsupported);
    assert_eq!(text(&t), "abZ");
    assert_eq!(t.pending_output_len(), 0);
    let (_, output, _) = partitions(b"\x1b]0;both\x07\x1b]1;icon\x07\x1b]2;\x07");
    assert_eq!(
        output,
        [
            OutputEvent::Title {
                target: TitleTarget::IconAndWindow,
                text: "both".into()
            },
            OutputEvent::Title {
                target: TitleTarget::Icon,
                text: "icon".into()
            },
            OutputEvent::Title {
                target: TitleTarget::Window,
                text: String::new()
            }
        ]
    );
}

#[test]
fn malformed_unsupported_and_cancelled_strings_never_produce_effects() {
    for input in [
        b"\x1b]52;c;SECRET\x07Z".as_slice(),
        b"\x1b]52;c;?\x07Z",
        b"\x1b]2;bad\xff\x07Z",
        b"\x1b]2;bad\xc2\x9b\x07Z",
        b"\x1b]2;bad\ntext\x07Z",
        b"\x1b]2;secret\x18Z",
        b"\x1b]2;secret\x1aZ",
        b"\x1b]2;secret\x1b[6n\x1b\\Z",
        b"\x1bP\x1b]2;secret\x07\x1b\\Z",
    ] {
        let (t, output, out) = partitions(input);
        assert!(out.unsupported);
        assert!(output.is_empty());
        assert_eq!(text(&t), "Z");
    }
    let mut t = terminal();
    t.feed(b"\x1b]2;unfinished");
    assert!(t.finish().unsupported);
    assert!(drain(&mut t).is_empty());
}

#[test]
fn queue_count_pressure_pauses_exactly_once_without_losing_input_or_events() {
    let mut t = terminal();
    let mut bytes = vec![7; MAX_QUEUED_EVENTS + 1];
    bytes.extend_from_slice(b"ABC");
    let progress = t.feed(&bytes);
    assert_eq!(progress.consumed, MAX_QUEUED_EVENTS + 1);
    assert!(progress.output_blocked);
    assert_eq!(t.pending_output_len(), MAX_QUEUED_EVENTS + 1);
    assert_eq!(text(&t), "");
    let before = t.clone();
    assert_eq!(t.feed(b"ABC").consumed, 0);
    assert!(t.finish().output_blocked);
    assert_eq!(t, before);
    assert_eq!(t.pop_output(), Some(OutputEvent::Bell));
    let resumed = t.feed(&bytes[progress.consumed..]);
    assert_eq!(resumed.consumed, 3);
    assert!(!resumed.output_blocked);
    assert_eq!(text(&t), "ABC");
    assert_eq!(drain(&mut t), vec![OutputEvent::Bell; MAX_QUEUED_EVENTS]);
}

#[test]
fn payload_pressure_preserves_the_waiting_title_and_bounds_total_retention() {
    let mut t = terminal();
    let title = "x".repeat(MAX_OSC_BYTES - 2);
    let command = format!("\x1b]2;{title}\x07");
    let mut bytes = command.repeat(9).into_bytes();
    bytes.push(b'Z');
    let progress = t.feed(&bytes);
    assert!(progress.output_blocked);
    assert_eq!(progress.consumed, command.len() * 9);
    assert_eq!(t.pending_output_len(), 9);
    assert_eq!(t.pending_output_bytes(), title.len() * 9);
    assert!(t.pending_output_bytes() <= MAX_QUEUED_PAYLOAD_BYTES + MAX_EVENT_PAYLOAD_BYTES);
    assert_eq!(
        drain(&mut t),
        vec![
            OutputEvent::Title {
                target: TitleTarget::Window,
                text: title
            };
            9
        ]
    );
    assert_eq!(t.feed(&bytes[progress.consumed..]).consumed, 1);
    assert_eq!(text(&t), "Z");
}

#[test]
fn delayed_replies_keep_query_time_cursor_across_resize_and_alternate_screen() {
    let mut t = Terminal::new(
        Size {
            columns: 4,
            lines: 2,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap();
    t.feed(b"abcd\x1b[6n");
    t.resize(Size {
        columns: 2,
        lines: 2,
    })
    .unwrap();
    t.feed(b"\x1b[6n\x1b[?1049h\x1b[6n\x1b[?1049l\x1b[6n");
    assert_eq!(
        drain(&mut t),
        [
            b"\x1b[1;4R".as_slice(),
            b"\x1b[2;2R",
            b"\x1b[1;1R",
            b"\x1b[2;2R"
        ]
        .map(|b| OutputEvent::Reply(b.to_vec()))
    );
}

#[test]
fn backpressure_during_a_csi_preserves_parser_state_through_finish_and_resume() {
    let mut t = terminal();
    t.feed(&[7; MAX_QUEUED_EVENTS]);
    let out = t.feed(b"\x1b[\x076nZ");
    assert_eq!(out.consumed, 3);
    assert!(out.output_blocked && t.finish().output_blocked);
    assert_eq!(
        drain(&mut t),
        vec![OutputEvent::Bell; MAX_QUEUED_EVENTS + 1]
    );
    assert_eq!(t.feed(b"6nZ").consumed, 3);
    assert_eq!(drain(&mut t), [OutputEvent::Reply(b"\x1b[1;1R".to_vec())]);
    assert_eq!(text(&t), "Z");
}

#[test]
fn mixed_output_flood_is_lossless_under_chunking_and_repeated_queue_pressure() {
    let input = b"x\x1b[5n\x07\x1b]2;title\x1b\\\r".repeat(100);
    let expected = stream(&[&input]);
    assert_eq!(expected.1.len(), 300);
    assert_eq!(expected.2.consumed, input.len());
    for n in [1, 2, 7, 64, 127] {
        assert_eq!(stream(&input.chunks(n).collect::<Vec<_>>()), expected);
    }
    for split in (0..input.len()).step_by(17) {
        assert_eq!(stream(&[&input[..split], &input[split..]]), expected);
    }
}
