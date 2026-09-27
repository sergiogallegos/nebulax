//! Engine-owned bounded key and paste encoding.
use crate::Terminal;

/// Maximum raw UTF-8 bytes in one paste. Larger pastes are rejected atomically.
pub const MAX_PASTE_BYTES: usize = 4096;
/// Start/end markers add at most twelve bytes to a validated paste.
pub const PASTE_FRAME_BYTES: usize = 12;

/// Plain text with TAB/CR/LF only; rejects ESC, DEL and other C0/C1 controls.
/// Rejecting (rather than stripping) controls prevents embedded end markers from
/// escaping the frame. Clipboard acquisition/confirmation belongs to the host.
pub fn valid_paste(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_PASTE_BYTES
        && !text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\t' | '\r' | '\n'))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Delete,
    Control(u8),
}
impl Key {
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            1 => Self::Enter,
            2 => Self::Backspace,
            3 => Self::Tab,
            4 => Self::Escape,
            5 => Self::Up,
            6 => Self::Down,
            7 => Self::Left,
            8 => Self::Right,
            9 => Self::Home,
            10 => Self::End,
            11 => Self::Delete,
            32..=63 => Self::Control((code - 32) as u8),
            _ => return None,
        })
    }
}
impl Terminal {
    /// Global mode, independent of screen/cursor saves; soft/hard reset clear it.
    pub fn bracketed_paste(&self) -> bool {
        self.bracketed_paste
    }

    /// Encode once on dispatch. LF and CRLF become CR; TAB and all other accepted
    /// text are preserved. No bytes are emitted for invalid or oversized input.
    /// The host must serialize the returned frame without interleaving writes.
    pub fn encode_paste(&self, text: &str) -> Option<Vec<u8>> {
        if !valid_paste(text) {
            return None;
        }
        let mut bytes = Vec::with_capacity(text.len() + PASTE_FRAME_BYTES);
        if self.bracketed_paste {
            bytes.extend_from_slice(b"\x1b[200~");
        }
        let mut previous_cr = false;
        for byte in text.bytes() {
            if byte != b'\n' || !previous_cr {
                bytes.push(if byte == b'\n' { b'\r' } else { byte });
            }
            previous_cr = byte == b'\r';
        }
        if self.bracketed_paste {
            bytes.extend_from_slice(b"\x1b[201~");
        }
        Some(bytes)
    }

    /// Encoded on the engine owner at dispatch time. Kitty keyboard, modified
    /// function keys remain unsupported.
    pub fn encode_key(&self, key: Key) -> Option<Vec<u8>> {
        Some(match key {
            Key::Enter => b"\r".to_vec(),
            Key::Backspace => vec![127],
            Key::Tab => vec![9],
            Key::Escape => vec![27],
            Key::Up => vec![27, if self.application_cursor { b'O' } else { b'[' }, b'A'],
            Key::Down => vec![27, if self.application_cursor { b'O' } else { b'[' }, b'B'],
            Key::Left => vec![27, if self.application_cursor { b'O' } else { b'[' }, b'D'],
            Key::Right => vec![27, if self.application_cursor { b'O' } else { b'[' }, b'C'],
            Key::Home => vec![27, if self.application_cursor { b'O' } else { b'[' }, b'H'],
            Key::End => vec![27, if self.application_cursor { b'O' } else { b'[' }, b'F'],
            Key::Delete => b"\x1b[3~".to_vec(),
            Key::Control(c @ 0..=31) => vec![c],
            Key::Control(_) => return None,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, Size, WidthPolicy};
    #[test]
    fn basic_keys_and_controls_have_explicit_normal_mode_encoding() {
        let t = Terminal::new(
            Size {
                columns: 8,
                lines: 3,
            },
            Limits::default(),
            WidthPolicy::default(),
        )
        .unwrap();
        for (code, expected) in [
            (1, b"\r".as_slice()),
            (2, b"\x7f"),
            (3, b"\t"),
            (4, b"\x1b"),
            (5, b"\x1b[A"),
            (6, b"\x1b[B"),
            (7, b"\x1b[D"),
            (8, b"\x1b[C"),
            (9, b"\x1b[H"),
            (10, b"\x1b[F"),
            (11, b"\x1b[3~"),
        ] {
            assert_eq!(
                t.encode_key(Key::from_code(code).unwrap()).unwrap(),
                expected
            );
        }
        for n in 0..32 {
            assert_eq!(
                t.encode_key(Key::from_code(32 + n).unwrap()).unwrap(),
                [n as u8]
            );
        }
        assert_eq!(Key::from_code(0), None);
        assert_eq!(Key::from_code(u32::MAX), None);
        assert_eq!(t.encode_key(Key::Control(32)), None);
    }
}
