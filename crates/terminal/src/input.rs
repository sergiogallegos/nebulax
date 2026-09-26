//! Initial normal-cursor keyboard profile; future input modes belong here.
use crate::Terminal;

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
    /// Encoded on the engine owner at dispatch time. Application-cursor, kitty
    /// keyboard, modified function keys and bracketed paste remain unsupported.
    pub fn encode_key(&self, key: Key) -> Option<Vec<u8>> {
        Some(match key {
            Key::Enter => b"\r".to_vec(),
            Key::Backspace => vec![127],
            Key::Tab => vec![9],
            Key::Escape => vec![27],
            Key::Up => b"\x1b[A".to_vec(),
            Key::Down => b"\x1b[B".to_vec(),
            Key::Left => b"\x1b[D".to_vec(),
            Key::Right => b"\x1b[C".to_vec(),
            Key::Home => b"\x1b[H".to_vec(),
            Key::End => b"\x1b[F".to_vec(),
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
