//! Bounded 7-bit VT syntax, separate from UTF-8 decoding and terminal meaning.
//! DCS/APC/PM/SOS are deliberately discarded, not interpreted in this slice.
pub const MAX_PARAMETERS: usize = 32;
pub const MAX_INTERMEDIATES: usize = 2;
pub const MAX_HEADER_BYTES: usize = 64;
pub const MAX_OSC_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Parameter {
    /// None preserves an omitted value; the semantic layer supplies defaults.
    pub value: Option<u16>,
    /// True if separated from the preceding value by ':' rather than ';'.
    pub subparameter: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Header {
    parameters: [Parameter; MAX_PARAMETERS],
    len: u8,
    prefix: Option<u8>,
    intermediates: [u8; MAX_INTERMEDIATES],
    intermediate_len: u8,
    bytes: u8,
}
impl Header {
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters[..usize::from(self.len)]
    }
    pub fn prefix(&self) -> Option<u8> {
        self.prefix
    }
    pub fn intermediates(&self) -> &[u8] {
        &self.intermediates[..usize::from(self.intermediate_len)]
    }
    fn byte(&mut self) -> bool {
        self.bytes += 1;
        usize::from(self.bytes) <= MAX_HEADER_BYTES
    }
    fn intermediate(&mut self, b: u8) -> bool {
        if usize::from(self.intermediate_len) == MAX_INTERMEDIATES {
            return false;
        }
        self.intermediates[usize::from(self.intermediate_len)] = b;
        self.intermediate_len += 1;
        true
    }
    fn parameter(&mut self, subparameter: bool) -> bool {
        if usize::from(self.len) == MAX_PARAMETERS {
            return false;
        }
        self.parameters[usize::from(self.len)] = Parameter {
            value: None,
            subparameter,
        };
        self.len += 1;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringKind {
    Osc,
    Dcs,
    Apc,
    Pm,
    Sos,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Print(u8),
    Execute(u8),
    /// A ground-state DEL closes the text extension target.
    Boundary,
    /// ESC defers the decision until dispatch, since SGR preserves graphemes.
    EscapeBoundary,
    Esc {
        header: Header,
        final_byte: u8,
    },
    Csi {
        header: Header,
        final_byte: u8,
    },
    Osc(Vec<u8>),
    IgnoredString(StringKind),
    Cancelled {
        string: bool,
    },
    Invalid,
    Limit,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape(Header),
    EscapeIgnore,
    Csi(Header),
    CsiIgnore,
    Osc {
        bytes: Vec<u8>,
        escape: bool,
        discard: bool,
    },
    IgnoreString {
        kind: StringKind,
        escape: bool,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parser {
    state: State,
}
impl Parser {
    pub fn is_ground(&self) -> bool {
        self.state == State::Ground
    }
    pub fn reset(&mut self) {
        self.state = State::Ground;
    }
    /// One input byte produces at most one event. Feed boundaries have no meaning.
    pub fn push(&mut self, b: u8) -> Option<Event> {
        use Event::*;
        if matches!(b, 0x18 | 0x1a) {
            let string = matches!(self.state, State::Osc { .. } | State::IgnoreString { .. });
            self.reset();
            return Some(Cancelled { string });
        }
        // Strings own their bytes through BEL (OSC), ST, or explicit cancellation.
        // A non-ST ESC never re-enters command parsing from an ignored payload.
        if let State::IgnoreString { escape, .. } = &mut self.state {
            if *escape && b == b'\\' {
                self.reset();
            } else {
                *escape = b == 0x1b;
            }
            return None;
        }
        if let State::Osc {
            bytes,
            escape,
            discard,
        } = &mut self.state
        {
            if b == 7 || (*escape && b == b'\\') {
                let valid = !*discard && !(*escape && b == 7);
                let event = if valid {
                    Osc(std::mem::take(bytes))
                } else {
                    IgnoredString(StringKind::Osc)
                };
                self.reset();
                return Some(event);
            }
            if *escape && !*discard {
                *discard = true;
                bytes.clear();
                *escape = b == 0x1b;
                return Some(Invalid);
            }
            *escape = b == 0x1b;
            if *escape || *discard {
                return None;
            }
            if b < 0x20 || b == 0x7f {
                *discard = true;
                bytes.clear();
                return Some(Invalid);
            }
            if bytes.len() == MAX_OSC_BYTES {
                *discard = true;
                bytes.clear();
                return Some(Limit);
            }
            bytes.push(b);
            return None;
        }
        if b == 0x1b {
            self.state = State::Escape(Header::default());
            return Some(EscapeBoundary);
        }
        if b < 0x20 {
            return Some(Execute(b));
        }
        if b == 0x7f {
            return self.is_ground().then_some(Boundary);
        }
        match std::mem::take(&mut self.state) {
            State::Ground => Some(Print(b)),
            State::Escape(mut h) => {
                if h.intermediate_len == 0 {
                    match b {
                        b'[' => {
                            self.state = State::Csi(Header::default());
                            return None;
                        }
                        b']' => {
                            self.state = State::Osc {
                                bytes: Vec::new(),
                                escape: false,
                                discard: false,
                            };
                            return None;
                        }
                        b'P' | b'_' | b'^' | b'X' => {
                            let kind = match b {
                                b'P' => StringKind::Dcs,
                                b'_' => StringKind::Apc,
                                b'^' => StringKind::Pm,
                                _ => StringKind::Sos,
                            };
                            self.state = State::IgnoreString {
                                kind,
                                escape: false,
                            };
                            return Some(IgnoredString(kind));
                        }
                        _ => {}
                    }
                }
                if (0x30..=0x7e).contains(&b) {
                    return Some(Esc {
                        header: h,
                        final_byte: b,
                    });
                }
                if (0x20..=0x2f).contains(&b) {
                    if h.byte() && h.intermediate(b) {
                        self.state = State::Escape(h);
                        None
                    } else {
                        self.state = State::EscapeIgnore;
                        Some(Limit)
                    }
                } else {
                    self.state = State::EscapeIgnore;
                    Some(Invalid)
                }
            }
            State::Csi(mut h) => {
                if (0x40..=0x7e).contains(&b) {
                    return Some(Csi {
                        header: h,
                        final_byte: b,
                    });
                }
                if !h.byte() {
                    self.state = State::CsiIgnore;
                    return Some(Limit);
                }
                let failure = if (0x20..=0x2f).contains(&b) {
                    (!h.intermediate(b)).then_some(Limit)
                } else if h.intermediate_len > 0 {
                    Some(Invalid)
                } else if (0x3c..=0x3f).contains(&b) {
                    if h.bytes == 1 {
                        h.prefix = Some(b);
                        None
                    } else {
                        Some(Invalid)
                    }
                } else if b.is_ascii_digit() {
                    if h.len == 0 {
                        h.parameter(false);
                    }
                    let value = &mut h.parameters[usize::from(h.len - 1)].value;
                    match value
                        .unwrap_or(0)
                        .checked_mul(10)
                        .and_then(|n| n.checked_add(u16::from(b - b'0')))
                    {
                        Some(n) => {
                            *value = Some(n);
                            None
                        }
                        None => Some(Limit),
                    }
                } else if matches!(b, b';' | b':') {
                    if h.len == 0 {
                        h.parameter(false);
                    }
                    (!h.parameter(b == b':')).then_some(Limit)
                } else {
                    Some(Invalid)
                };
                if let Some(event) = failure {
                    self.state = State::CsiIgnore;
                    Some(event)
                } else {
                    self.state = State::Csi(h);
                    None
                }
            }
            State::CsiIgnore => {
                if !(0x40..=0x7e).contains(&b) {
                    self.state = State::CsiIgnore;
                }
                None
            }
            State::EscapeIgnore => {
                if !(0x30..=0x7e).contains(&b) {
                    self.state = State::EscapeIgnore;
                }
                None
            }
            State::Osc { .. } | State::IgnoreString { .. } => unreachable!(),
        }
    }
}
