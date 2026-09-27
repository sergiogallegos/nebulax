//! Explicit research identification profile and state-derived protocol replies.
use crate::{Terminal, parser::Header};

#[derive(Clone, Copy)]
pub(crate) enum Query {
    Status,
    Cursor,
    PrimaryAttributes,
    SecondaryAttributes,
    Version,
    Mode { private: bool, number: u16 },
}
impl Query {
    pub fn parse(header: Header, final_byte: u8) -> Option<Self> {
        let params = header.parameters();
        if params.len() > 1 || params.iter().any(|p| p.subparameter) {
            return None;
        }
        let value = params.first().and_then(|p| p.value);
        if header.intermediates() == b"$" && final_byte == b'p' {
            return match header.prefix() {
                None | Some(b'?') => Some(Self::Mode {
                    private: header.prefix().is_some(),
                    number: value?,
                }),
                _ => None,
            };
        }
        if !header.intermediates().is_empty() {
            return None;
        }
        match (header.prefix(), value.unwrap_or(0), final_byte) {
            (None, 5, b'n') => Some(Self::Status),
            (None, 6, b'n') => Some(Self::Cursor),
            (None, 0, b'c') => Some(Self::PrimaryAttributes),
            (Some(b'>'), 0, b'c') => Some(Self::SecondaryAttributes),
            (Some(b'>'), 0, b'q') => Some(Self::Version),
            _ => None,
        }
    }
    pub fn reply(self, terminal: &Terminal) -> Vec<u8> {
        match self {
            Self::Status => b"\x1b[0n".to_vec(),
            Self::Cursor => {
                let cursor = terminal.cursor();
                let origin = if terminal.active.origin {
                    terminal.active.top
                } else {
                    0
                };
                format!("\x1b[{};{}R", cursor.row + 1 - origin, cursor.column + 1).into_bytes()
            }
            // Minimal VT100-family compatibility profile, no option bits.
            // DA2 revision 1 is our profile revision, never an xterm patch level.
            Self::PrimaryAttributes => b"\x1b[?1;0c".to_vec(),
            Self::SecondaryAttributes => b"\x1b[>0;1;0c".to_vec(),
            Self::Version => concat!(
                "\x1bP>|Nebulax ",
                env!("CARGO_PKG_VERSION"),
                " (experimental)\x1b\\"
            )
            .as_bytes()
            .to_vec(),
            Self::Mode { private, number } => {
                let value = if private {
                    match number {
                        1 => Some(terminal.application_cursor),
                        2004 => Some(terminal.bracketed_paste),
                        6 => Some(terminal.active.origin),
                        7 => Some(terminal.autowrap()),
                        25 => Some(terminal.cursor_visible()),
                        1049 => Some(terminal.is_alternate()),
                        _ => None,
                    }
                } else {
                    (number == 4).then_some(terminal.insert_mode())
                };
                let state = match value {
                    None => 0,
                    Some(true) => 1,
                    Some(false) => 2,
                };
                format!("\x1b[{}{number};{state}$y", if private { "?" } else { "" }).into_bytes()
            }
        }
    }
}
