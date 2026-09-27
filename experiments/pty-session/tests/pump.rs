use nebulax_pty_session::{Pump, READ_CAPACITY, Step};
use nebulax_terminal::{CellView as Cell, Limits, OutputEvent, Size, Terminal, WidthPolicy};
use std::collections::VecDeque;
use std::io::{self, Read, Write};

fn engine() -> Terminal {
    Terminal::new(
        Size {
            columns: 20,
            lines: 3,
        },
        Limits::default(),
        WidthPolicy::default(),
    )
    .unwrap()
}
struct Wire {
    input: VecDeque<u8>,
    output: Vec<u8>,
    reads: usize,
    read_chunk: usize,
    write_chunk: usize,
    write_blocked: bool,
    write_budget: usize,
    interrupt_read: bool,
    interrupt_write: bool,
    eof: bool,
    zero_write: bool,
    broken_write: bool,
}
impl Wire {
    fn new(input: &[u8]) -> Self {
        Self {
            input: input.iter().copied().collect(),
            output: Vec::new(),
            reads: 0,
            read_chunk: READ_CAPACITY,
            write_chunk: usize::MAX,
            write_blocked: false,
            write_budget: usize::MAX,
            interrupt_read: false,
            interrupt_write: false,
            eof: true,
            zero_write: false,
            broken_write: false,
        }
    }
}
impl Read for Wire {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.reads += 1;
        if std::mem::take(&mut self.interrupt_read) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.input.is_empty() && !self.eof {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let n = out.len().min(self.input.len()).min(self.read_chunk);
        for b in &mut out[..n] {
            *b = self.input.pop_front().unwrap();
        }
        Ok(n)
    }
}
impl Write for Wire {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if std::mem::take(&mut self.interrupt_write) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.write_blocked || self.write_budget == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        if self.broken_write {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        if self.zero_write {
            return Ok(0);
        }
        let n = input.len().min(self.write_chunk).min(self.write_budget);
        self.write_budget -= n;
        self.output.extend_from_slice(&input[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn text(pump: &Pump) -> String {
    pump.terminal()
        .screen()
        .iter()
        .flat_map(|r| r.cells())
        .filter_map(|c| match c.view() {
            Cell::Lead { cluster, .. } => Some(cluster.chars().collect::<String>()),
            _ => None,
        })
        .collect()
}
#[test]
fn blocked_partial_replies_resume_without_rereading_or_reordering() {
    let mut wire = Wire::new(b"ab\x1b[6n\x1b[5n\x1b]2;t\x07\x07Z");
    wire.write_blocked = true;
    let mut pump = Pump::new(engine());
    assert_eq!(
        pump.step(&mut wire, |_| panic!("reply must precede effects"))
            .unwrap(),
        Step::WriteBlocked
    );
    let reads = wire.reads;
    for _ in 0..3 {
        assert_eq!(pump.step(&mut wire, |_| false).unwrap(), Step::WriteBlocked);
        assert_eq!(wire.reads, reads);
    }
    wire.write_blocked = false;
    wire.write_chunk = 1;
    wire.interrupt_write = true;
    let mut effects = Vec::new();
    assert_eq!(
        pump.step(&mut wire, |e| {
            effects.push(e.clone());
            true
        })
        .unwrap(),
        Step::Finished
    );
    assert_eq!(wire.output, b"\x1b[1;3R\x1b[0n");
    assert!(
        matches!(&effects[..], [OutputEvent::Title { text, .. }, OutputEvent::Bell] if text == "t")
    );
    assert_eq!(text(&pump), "abZ");
}
#[test]
fn effect_pressure_stops_reads_and_preserves_every_event_in_a_flood() {
    let input = b"\x1b]2;t\x07\x07\x1b[5n".repeat(1000);
    let mut wire = Wire::new(&input);
    let mut pump = Pump::new(engine());
    assert_eq!(
        pump.step(&mut wire, |_| false).unwrap(),
        Step::EffectBlocked
    );
    let reads = wire.reads;
    for _ in 0..10 {
        assert_eq!(
            pump.step(&mut wire, |_| false).unwrap(),
            Step::EffectBlocked
        );
        assert_eq!(wire.reads, reads);
        assert!(pump.retained_input() <= READ_CAPACITY);
        assert!(pump.retained_event_bytes() <= 1024);
        assert!(pump.terminal().pending_output_len() <= 65);
        assert!(pump.terminal().pending_output_bytes() <= 9216);
    }
    let mut count = 0;
    for _ in 0..200 {
        let step = pump
            .step(&mut wire, |e| {
                assert_eq!(matches!(e, OutputEvent::Title { .. }), count % 2 == 0);
                count += 1;
                true
            })
            .unwrap();
        if step == Step::Finished {
            break;
        }
    }
    assert!(pump.is_finished());
    assert_eq!(count, 2000);
    assert_eq!(wire.output, b"\x1b[0n".repeat(1000));
}
#[test]
fn interrupted_and_would_block_reads_do_not_flush_partial_utf8_or_csi() {
    let mut wire = Wire::new(b"\xf0\x9f");
    wire.interrupt_read = true;
    wire.eof = false;
    wire.read_chunk = 1;
    let mut pump = Pump::new(engine());
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::ReadBlocked);
    assert_eq!(text(&pump), "");
    wire.input.extend(b"\x91\xa9\x1b[6");
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::ReadBlocked);
    assert_eq!(text(&pump), "👩");
    wire.input.extend(b"n\xf0\x9f");
    wire.eof = true;
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::Finished);
    assert_eq!(wire.output, b"\x1b[1;3R");
    assert_eq!(text(&pump), "👩�");
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::Finished);
    assert_eq!(text(&pump), "👩�");
}
#[test]
fn zero_or_failed_writes_are_errors_with_the_event_still_owned() {
    for broken in [false, true] {
        let mut wire = Wire::new(b"\x1b[5n");
        wire.zero_write = !broken;
        wire.broken_write = broken;
        let mut pump = Pump::new(engine());
        assert_eq!(
            pump.step(&mut wire, |_| true).unwrap_err().kind(),
            if broken {
                io::ErrorKind::BrokenPipe
            } else {
                io::ErrorKind::WriteZero
            }
        );
        assert_eq!(pump.retained_event_bytes(), 4);
        wire.zero_write = false;
        wire.broken_write = false;
        assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::Finished);
        assert_eq!(wire.output, b"\x1b[0n");
    }
}

#[test]
fn would_block_after_each_partial_write_preserves_offset_and_effect_order() {
    let mut wire = Wire::new(b"ab\x1b[6n\x07\x1b[5n");
    let mut pump = Pump::new(engine());
    let expected = b"\x1b[1;3R\x1b[0n";
    let mut bells = 0;
    for n in 1..expected.len() {
        wire.write_budget = 1;
        assert_eq!(
            pump.step(&mut wire, |event| {
                assert_eq!(event, &OutputEvent::Bell);
                assert!(n >= 6); // First reply has been fully written before the bell.
                bells += 1;
                true
            })
            .unwrap(),
            Step::WriteBlocked
        );
        assert_eq!(wire.output, expected[..n]);
    }
    wire.write_budget = 1;
    assert_eq!(
        pump.step(&mut wire, |_| panic!("bell already consumed"))
            .unwrap(),
        Step::Finished
    );
    assert_eq!(wire.output, expected);
    assert_eq!(bells, 1);
}

#[test]
fn native_input_never_splits_a_reply_and_preserves_utf8_and_key_order() {
    use nebulax_pty_session::input::{Input, InputError};
    use nebulax_terminal::input::Key;
    let mut wire = Wire::new(b"\x1b[5n");
    wire.eof = false;
    wire.write_budget = 1;
    let mut pump = Pump::new(engine());
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::WriteBlocked);
    assert_eq!(wire.output, b"\x1b");
    pump.queue_input(Input::Text("界".into())).unwrap();
    assert_eq!(pump.queue_input(Input::Key(Key::Up)), Err(InputError::Full));
    let expected = b"\x1b[0n\xe7\x95\x8c\x1b[A";
    let mut key_queued = false;
    for n in 2..=expected.len() {
        wire.write_budget = 1;
        pump.step(&mut wire, |_| true).unwrap();
        assert_eq!(wire.output, expected[..n]);
        if !key_queued && pump.can_accept_input() {
            // Enqueue another event during a partially written UTF-8 commit.
            pump.queue_input(Input::Key(Key::Up)).unwrap();
            key_queued = true;
        }
    }
    assert!(key_queued);
    wire.eof = true;
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::Finished);
    assert_eq!(
        pump.queue_input(Input::Text("late".into())),
        Err(InputError::Closed)
    );
}

#[test]
fn bounded_echo_is_drained_during_a_partial_input_write_to_avoid_deadlock() {
    use nebulax_pty_session::input::Input;
    struct Echo {
        readable: VecDeque<u8>,
        written: Vec<u8>,
    }
    impl Read for Echo {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.readable.is_empty() {
                return Err(io::ErrorKind::WouldBlock.into());
            }
            let n = out.len().min(self.readable.len());
            for byte in &mut out[..n] {
                *byte = self.readable.pop_front().unwrap();
            }
            Ok(n)
        }
    }
    impl Write for Echo {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let n = bytes.len().min(4 - self.readable.len());
            if n == 0 {
                return Err(io::ErrorKind::WouldBlock.into());
            }
            self.readable.extend(&bytes[..n]);
            self.written.extend_from_slice(&bytes[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut wire = Echo {
        readable: VecDeque::new(),
        written: Vec::new(),
    };
    let mut pump = Pump::new(engine());
    pump.queue_input(Input::Text("abcdefghijklmnop".into()))
        .unwrap();
    for _ in 0..16 {
        pump.step(&mut wire, |_| true).unwrap();
    }
    assert_eq!(wire.written, b"abcdefghijklmnop");
    assert_eq!(text(&pump), "abcdefghijklmnop");
    assert_eq!(pump.retained_event_bytes(), 0);
}

#[test]
fn queued_keys_use_dispatch_time_modes_but_partial_key_writes_are_immutable() {
    use nebulax_pty_session::input::Input;
    use nebulax_terminal::input::Key;
    let mut wire = Wire::new(b"\x1b[5n");
    wire.eof = false;
    wire.write_budget = 1;
    let mut pump = Pump::new(engine());
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::WriteBlocked);
    pump.queue_input(Input::Key(Key::Up)).unwrap();
    wire.input.extend(b"\x1b[?1h");
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::Yield);
    wire.write_budget = 64;
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::ReadBlocked);
    assert_eq!(wire.output, b"\x1b[0n\x1bOA");

    pump.queue_input(Input::Key(Key::Down)).unwrap();
    wire.write_budget = 1;
    wire.input.extend(b"\x1b[?1l");
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::Yield);
    pump.queue_input(Input::Key(Key::Left)).unwrap();
    wire.write_budget = 64;
    assert_eq!(pump.step(&mut wire, |_| true).unwrap(), Step::ReadBlocked);
    assert_eq!(wire.output, b"\x1b[0n\x1bOA\x1bOB\x1b[D");
    assert!(!pump.diagnostics().unsupported);
}
