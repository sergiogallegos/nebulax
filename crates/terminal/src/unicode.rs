//! Original streaming extended-grapheme rules from UAX #29 revision 49.
//! Width is a separate terminal policy, not a Unicode conformance claim.
use crate::tables::PROPERTIES;

const CR: u16 = 1;
const LF: u16 = 2;
const CONTROL: u16 = 3;
const EXTEND: u16 = 4;
const ZWJ: u16 = 5;
const RI: u16 = 6;
const PREPEND: u16 = 7;
const SPACING: u16 = 8;
const L: u16 = 9;
const V: u16 = 10;
const T: u16 = 11;
const LV: u16 = 12;
const LVT: u16 = 13;

fn properties(c: char) -> u16 {
    let cp = u32::from(c);
    let i = PROPERTIES.partition_point(|&(_, end, _)| end < cp);
    PROPERTIES
        .get(i)
        .filter(|&&(start, _, _)| start <= cp)
        .map_or(0, |&(_, _, p)| p)
}

/// Constant-space Unicode 18 extended-grapheme boundary detector.
/// Feed every scalar, including controls. `push` reports a boundary before it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GraphemeBreak {
    previous: Option<u16>,
    ri_odd: bool,
    pictographic_extend: bool,
    pictographic_zwj: bool,
    indic_linker: bool,
}

impl GraphemeBreak {
    pub fn push(&mut self, c: char) -> bool {
        let p = properties(c);
        let next = p & 15;
        let incb = (p >> 4) & 3;
        let ep = p & (1 << 6) != 0;
        let boundary = match self.previous {
            None => true,                                         // GB1
            Some(CR) if next == LF => false,                      // GB3
            Some(CR | LF | CONTROL) => true,                      // GB4
            Some(_) if matches!(next, CR | LF | CONTROL) => true, // GB5
            Some(L) if matches!(next, L | V | LV | LVT) => false,
            Some(LV | V) if matches!(next, V | T) => false,
            Some(LVT | T) if next == T => false,
            Some(_) if matches!(next, EXTEND | ZWJ | SPACING) => false,
            Some(PREPEND) => false,
            Some(_) if self.indic_linker && incb == 1 => false, // GB9c (Unicode 18)
            Some(_) if self.pictographic_zwj && ep => false,    // GB11
            Some(RI) if next == RI && self.ri_odd => false,     // GB12/13
            Some(_) => true,
        };
        self.ri_odd = next == RI && (self.previous != Some(RI) || !self.ri_odd);
        self.pictographic_zwj = next == ZWJ && self.pictographic_extend;
        self.pictographic_extend = ep || (next == EXTEND && self.pictographic_extend);
        self.indic_linker = incb == 2 || (incb == 3 && self.indic_linker);
        self.previous = Some(next);
        boundary
    }
}

/// Experimental terminal width profile: emoji presentation is wide, text
/// presentation is narrow; ambiguous EAW characters are configurable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidthPolicy {
    pub ambiguous_wide: bool,
}

impl WidthPolicy {
    pub(crate) fn scalar(self, c: char) -> u8 {
        let p = properties(c);
        if matches!(p & 15, CR | LF | CONTROL | EXTEND | ZWJ | PREPEND | SPACING)
            || p & (1 << 10) != 0
        {
            0
        } else if p & ((1 << 7) | (1 << 8)) != 0 || (self.ambiguous_wide && p & (1 << 9) != 0) {
            2
        } else {
            1
        }
    }

    pub(crate) fn extend(self, old: u8, previous: char, next: char) -> u8 {
        if properties(previous) & (1 << 11) != 0 {
            if next == '\u{fe0f}' {
                return 2;
            }
            if next == '\u{fe0e}' {
                return 1;
            }
        }
        old.max(self.scalar(next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_unicode_18_extended_grapheme_tests() {
        let data = include_str!("../../../third_party/unicode/18.0.0/GraphemeBreakTest.txt");
        let mut cases = 0;
        for (line, value) in data.lines().enumerate() {
            let tokens: Vec<_> = value
                .split('#')
                .next()
                .unwrap()
                .split_whitespace()
                .collect();
            if tokens.is_empty() {
                continue;
            }
            let mut detector = GraphemeBreak::default();
            for pair in tokens[..tokens.len() - 1].as_chunks::<2>().0 {
                let c = char::from_u32(u32::from_str_radix(pair[1], 16).unwrap()).unwrap();
                assert_eq!(
                    detector.push(c),
                    pair[0] == "÷",
                    "test line {}: {value}",
                    line + 1
                );
            }
            assert_eq!(tokens.last(), Some(&"÷")); // GB2 is an explicit stream end.
            cases += 1;
        }
        assert!(cases > 700, "conformance corpus unexpectedly truncated");
        eprintln!("Unicode 18.0.0: {cases} official grapheme cases passed");
    }
}
