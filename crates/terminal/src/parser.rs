//! Bounded subset parser. Unsupported strings are consumed without storing payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Control(u8),
    Left(usize),
    Erase(usize),
    EraseLine,
    Alternate(bool),
    Unsupported,
    Limit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Parser {
    #[default]
    Ground,
    Escape,
    EscapeIntermediate,
    Csi {
        value: u16,
        bytes: u8,
        unsupported: bool,
        discard: bool,
        private: bool,
    },
    String {
        osc: bool,
        escape: bool,
    },
}

impl Parser {
    pub fn push(&mut self, b: u8) -> Action {
        use Action::*;
        if b == 0x18 || b == 0x1a {
            *self = Self::Ground;
            return None;
        }
        if let Self::String { osc, escape } = *self {
            *self = if (osc && b == 7) || (escape && b == b'\\') {
                Self::Ground
            } else {
                Self::String {
                    osc,
                    escape: b == 0x1b,
                }
            };
            return None;
        }
        if b == 0x1b {
            *self = Self::Escape;
            return None;
        }
        if b < 0x20 {
            return Control(b);
        }
        if b == 0x7f {
            return None;
        }
        match *self {
            Self::Ground => Unsupported,
            Self::Escape => {
                *self = match b {
                    b'[' => Self::Csi {
                        value: 0,
                        bytes: 0,
                        unsupported: false,
                        discard: false,
                        private: false,
                    },
                    b']' | b'P' | b'X' | b'^' | b'_' => Self::String {
                        osc: b == b']',
                        escape: false,
                    },
                    0x20..=0x2f => Self::EscapeIntermediate,
                    _ => Self::Ground,
                };
                if b == b'[' { None } else { Unsupported }
            }
            Self::EscapeIntermediate => {
                if !(0x20..=0x2f).contains(&b) {
                    *self = Self::Ground;
                }
                None
            }
            Self::Csi {
                mut value,
                mut bytes,
                mut unsupported,
                mut discard,
                mut private,
            } => {
                if (0x40..=0x7e).contains(&b) {
                    *self = Self::Ground;
                    return if discard {
                        None
                    } else if unsupported {
                        Unsupported
                    } else if private {
                        match (value, b) {
                            (1049, b'h') => Alternate(true),
                            (1049, b'l') => Alternate(false),
                            _ => Unsupported,
                        }
                    } else {
                        match b {
                            b'D' => Left(usize::from(value.max(1))),
                            b'X' => Erase(usize::from(value.max(1))),
                            b'K' if value == 0 => EraseLine,
                            _ => Unsupported,
                        }
                    };
                }
                if discard {
                    return None;
                }
                if b == b'?' && bytes == 0 {
                    private = true;
                } else if b.is_ascii_digit() && !unsupported {
                    if let Some(n) = value
                        .checked_mul(10)
                        .and_then(|n| n.checked_add(u16::from(b - b'0')))
                    {
                        value = n;
                    } else {
                        discard = true;
                    }
                } else {
                    unsupported = true;
                }
                bytes += 1;
                discard |= bytes >= 64;
                *self = Self::Csi {
                    value,
                    bytes,
                    unsupported,
                    discard,
                    private,
                };
                if discard { Limit } else { None }
            }
            Self::String { .. } => unreachable!(),
        }
    }
}
