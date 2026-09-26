#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Decoder {
    bytes: [u8; 4],
    len: usize,
    needed: usize,
}

impl Decoder {
    pub fn push(&mut self, byte: u8) -> [Option<char>; 2] {
        if self.len == 0 {
            return [self.start(byte), None];
        }
        let valid = (0x80..=0xbf).contains(&byte)
            && (self.len != 1
                || match self.bytes[0] {
                    0xe0 => byte >= 0xa0,
                    0xed => byte <= 0x9f,
                    0xf0 => byte >= 0x90,
                    0xf4 => byte <= 0x8f,
                    _ => true,
                });
        if !valid {
            self.len = 0;
            return [Some('\u{fffd}'), self.start(byte)];
        }
        self.bytes[self.len] = byte;
        self.len += 1;
        if self.len != self.needed {
            return [None, None];
        }
        let c = std::str::from_utf8(&self.bytes[..self.len])
            .expect("validated UTF-8")
            .chars()
            .next();
        self.len = 0;
        [c, None]
    }

    fn start(&mut self, byte: u8) -> Option<char> {
        self.needed = match byte {
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            0..=0x7f => return Some(char::from(byte)),
            _ => return Some('\u{fffd}'),
        };
        self.bytes[0] = byte;
        self.len = 1;
        None
    }

    pub fn finish(&mut self) -> Option<char> {
        let pending = self.len != 0;
        self.len = 0;
        pending.then_some('\u{fffd}')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_prefixes_match_lossy_utf8_reference() {
        let alphabet = [
            0, 0x1b, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0, 0xc2, 0xdf, 0xe0, 0xed,
            0xef, 0xf0, 0xf4, 0xf5, 0xff,
        ];
        for a in alphabet {
            for b in alphabet {
                for c in alphabet {
                    let input = [a, b, c];
                    let mut decoder = Decoder::default();
                    let mut text = String::new();
                    for byte in input {
                        text.extend(decoder.push(byte).into_iter().flatten());
                    }
                    text.extend(decoder.finish());
                    assert_eq!(text, String::from_utf8_lossy(&input), "{input:x?}");
                }
            }
        }
    }
}
