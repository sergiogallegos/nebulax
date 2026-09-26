use nebulax_terminal::{FeedOutcome, OutputEvent, Terminal};
use std::io::{self, Read, Write};

pub const READ_CAPACITY: usize = 4096;
const TURN_OPERATIONS: usize = 64;

pub trait Transport: Read + Write {}
impl<T: Read + Write> Transport for T {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    ReadBlocked,
    WriteBlocked,
    EffectBlocked,
    Yield,
    /// All PTY input and terminal output drained, independently of child exit.
    Finished,
}

/// One authoritative engine, one fixed input buffer, one owned output event.
/// No callbacks run in the engine. The caller explicitly accepts or defers effects.
pub struct Pump {
    pub(crate) terminal: Terminal,
    input: [u8; READ_CAPACITY],
    start: usize,
    end: usize,
    pending: Option<OutputEvent>,
    written: usize,
    native_input: Option<crate::input::Input>,
    eof: bool,
    finished: bool,
    diagnostics: FeedOutcome,
    pub(crate) changed: bool,
}
impl Pump {
    pub fn new(terminal: Terminal) -> Self {
        Self {
            terminal,
            input: [0; READ_CAPACITY],
            start: 0,
            end: 0,
            pending: None,
            written: 0,
            native_input: None,
            eof: false,
            finished: false,
            diagnostics: FeedOutcome::default(),
            changed: true,
        }
    }
    pub fn can_accept_input(&self) -> bool {
        self.native_input.is_none() && !self.eof
    }
    pub fn queue_input(
        &mut self,
        input: crate::input::Input,
    ) -> Result<(), crate::input::InputError> {
        if !input.valid() {
            return Err(crate::input::InputError::Invalid);
        }
        if self.eof {
            return Err(crate::input::InputError::Closed);
        }
        if self.native_input.is_some() {
            return Err(crate::input::InputError::Full);
        }
        self.native_input = Some(input);
        Ok(())
    }
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }
    pub fn terminal(&self) -> &Terminal {
        &self.terminal
    }
    pub fn diagnostics(&self) -> FeedOutcome {
        self.diagnostics
    }
    pub fn retained_input(&self) -> usize {
        self.end - self.start
    }
    pub fn retained_event_bytes(&self) -> usize {
        self.pending.as_ref().map_or(0, OutputEvent::payload_len)
    }
    pub fn is_finished(&self) -> bool {
        self.finished && self.pending.is_none() && self.terminal.pending_output_len() == 0
    }
    // PTY echo can fill the output side while input is partially written. Drain
    // bounded readable bytes without replacing the current write/offset, or the
    // two sides can deadlock. Generated replies remain in the engine's bounded
    // queue; if that fills, its consumed count naturally stops further reads.
    fn read_while_write_blocked(&mut self, transport: &mut impl Transport) -> io::Result<Step> {
        if self.start == self.end && !self.eof {
            match transport.read(&mut self.input) {
                Ok(n) => {
                    self.start = 0;
                    self.end = n;
                    self.eof = n == 0;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => return Ok(Step::Yield),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(Step::WriteBlocked),
                Err(e) => return Err(e),
            }
        }
        if self.start < self.end {
            let out = self.terminal.feed(&self.input[self.start..self.end]);
            self.start += out.consumed;
            self.changed |= out.changed;
            self.diagnostics.merge(out);
            if out.consumed > 0 {
                return Ok(Step::Yield);
            }
        }
        Ok(Step::WriteBlocked)
    }
    /// At most 64 operations; transport must be nonblocking. A false effect
    /// result means no acceptance: the same owned event will be offered again.
    /// The caller must not apply an effect and then return false.
    pub fn step(
        &mut self,
        transport: &mut impl Transport,
        mut accept_effect: impl FnMut(&OutputEvent) -> bool,
    ) -> io::Result<Step> {
        for _ in 0..TURN_OPERATIONS {
            if self.pending.is_none() {
                self.pending = self.terminal.pop_output();
                if self.pending.is_none()
                    && let Some(input) = self.native_input.take()
                {
                    let bytes = match input {
                        crate::input::Input::Text(text) => text.into_bytes(),
                        crate::input::Input::Key(key) => {
                            self.terminal.encode_key(key).expect("validated key")
                        }
                    };
                    // A single writer/offset owns replies and native input alike.
                    // Complete it before selecting any other event; no byte interleaving.
                    self.pending = Some(OutputEvent::Reply(bytes));
                }
                self.written = 0;
            }
            if let Some(event) = &self.pending {
                match event {
                    OutputEvent::Reply(bytes) => match transport.write(&bytes[self.written..]) {
                        Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                        Ok(n) => {
                            self.written += n;
                            if self.written < bytes.len() {
                                continue;
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            return self.read_while_write_blocked(transport);
                        }
                        Err(e) => return Err(e),
                    },
                    _ if !accept_effect(event) => return Ok(Step::EffectBlocked),
                    _ => {}
                }
                self.pending = None;
                continue;
            }
            if self.start < self.end {
                let out = self.terminal.feed(&self.input[self.start..self.end]);
                self.start += out.consumed;
                self.changed |= out.changed;
                self.diagnostics.merge(out);
                continue;
            }
            if self.eof {
                if !self.finished {
                    let out = self.terminal.finish();
                    self.finished = !out.output_blocked;
                    self.changed |= out.changed;
                    self.diagnostics.merge(out);
                }
                if self.is_finished() {
                    return Ok(Step::Finished);
                }
                continue;
            }
            match transport.read(&mut self.input) {
                Ok(n) => {
                    self.start = 0;
                    self.end = n;
                    self.eof = n == 0;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(Step::ReadBlocked),
                Err(e) => return Err(e),
            }
        }
        Ok(Step::Yield)
    }
}
