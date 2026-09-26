//! Bounded output, inert until the caller applies runtime policy or writes a PTY.
use std::collections::VecDeque;

pub const MAX_QUEUED_EVENTS: usize = 64;
pub const MAX_QUEUED_PAYLOAD_BYTES: usize = 8192;
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleTarget {
    Icon,
    Window,
    IconAndWindow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputEvent {
    /// Only generated protocol response bytes; never an arbitrary echo of input.
    Reply(Vec<u8>),
    Bell,
    /// Untrusted text, not permission to execute a native operation.
    Title {
        target: TitleTarget,
        text: String,
    },
}
impl OutputEvent {
    pub fn payload_len(&self) -> usize {
        match self {
            Self::Reply(bytes) => bytes.len(),
            Self::Bell => 0,
            Self::Title { text, .. } => text.len(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Output {
    queue: VecDeque<OutputEvent>,
    bytes: usize,
    // One completed event may wait outside the full queue. No later input is
    // processed until it has been queued, preserving order with bounded storage.
    pending: Option<OutputEvent>,
}
impl Output {
    fn fits(&self, e: &OutputEvent) -> bool {
        self.queue.len() < MAX_QUEUED_EVENTS
            && self.bytes + e.payload_len() <= MAX_QUEUED_PAYLOAD_BYTES
    }
    pub fn emit(&mut self, e: OutputEvent) {
        assert!(self.pending.is_none() && e.payload_len() <= MAX_EVENT_PAYLOAD_BYTES);
        if self.fits(&e) {
            self.bytes += e.payload_len();
            self.queue.push_back(e);
        } else {
            self.pending = Some(e);
        }
    }
    pub fn blocked(&self) -> bool {
        self.pending.is_some()
    }
    pub fn len(&self) -> usize {
        self.queue.len() + usize::from(self.pending.is_some())
    }
    pub fn bytes(&self) -> usize {
        self.bytes + self.pending.as_ref().map_or(0, OutputEvent::payload_len)
    }
    pub fn pop(&mut self) -> Option<OutputEvent> {
        let e = self.queue.pop_front()?;
        self.bytes -= e.payload_len();
        if self.pending.as_ref().is_some_and(|e| self.fits(e)) {
            let waiting = self.pending.take().unwrap();
            self.bytes += waiting.payload_len();
            self.queue.push_back(waiting);
        }
        Some(e)
    }
    pub fn valid(&self) -> bool {
        self.queue.len() <= MAX_QUEUED_EVENTS
            && self.bytes <= MAX_QUEUED_PAYLOAD_BYTES
            && self.bytes
                == self
                    .queue
                    .iter()
                    .map(OutputEvent::payload_len)
                    .sum::<usize>()
            && self
                .queue
                .iter()
                .chain(self.pending.iter())
                .all(|e| e.payload_len() <= MAX_EVENT_PAYLOAD_BYTES)
    }
}
