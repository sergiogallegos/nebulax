//! Bounded rendition values and original SGR interpretation.
use crate::{Terminal, parser::Header};
pub const MAX_STYLES: usize = 1024;
pub const BOLD: u32 = 1;
pub const FAINT: u32 = 2;
pub const ITALIC: u32 = 4;
pub const UNDERLINE: u32 = 8;
pub const INVERSE: u32 = 16;
pub const HIDDEN: u32 = 32;
pub const STRIKE: u32 = 64;
pub const DOUBLE_UNDERLINE: u32 = 128;

/// Wire colors: zero = default, 0x01000000|index, 0x02000000|RRGGBB.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub foreground: u32,
    pub background: u32,
    pub attributes: u32,
}
impl Style {
    pub const fn indexed(index: u8) -> u32 {
        0x0100_0000 | index as u32
    }
    pub const fn rgb(red: u8, green: u8, blue: u8) -> u32 {
        0x0200_0000 | (red as u32) << 16 | (green as u32) << 8 | blue as u32
    }
    pub(crate) fn erased(self) -> Self {
        Self {
            background: self.background,
            ..Self::default()
        }
    }
    pub(crate) fn sgr(mut self, header: Header) -> Option<Self> {
        let p = header.parameters();
        if p.is_empty() {
            return Some(Self::default());
        }
        let value = |i: usize| p.get(i)?.value;
        let component = |i: usize| u8::try_from(value(i)?).ok();
        let mut i = 0;
        while i < p.len() {
            if p[i].subparameter {
                return None;
            }
            let code = p[i].value.unwrap_or(0);
            let mut end = i + 1;
            while end < p.len() && p[end].subparameter {
                end += 1;
            }
            if matches!(code, 38 | 48) {
                let colon = end > i + 1;
                let mode = value(i + 1)?;
                let count = match mode {
                    5 => 3,
                    2 => {
                        if colon && end - i == 6 {
                            6
                        } else {
                            5
                        }
                    }
                    _ => return None,
                };
                if colon {
                    if end - i != count {
                        return None;
                    }
                } else {
                    end = i + count;
                    if end > p.len() || p[i + 1..end].iter().any(|v| v.subparameter) {
                        return None;
                    }
                }
                let color = if mode == 5 {
                    Self::indexed(component(i + 2)?)
                } else {
                    let offset = if count == 6 {
                        if !matches!(p[i + 2].value, None | Some(0)) {
                            return None;
                        }
                        3
                    } else {
                        2
                    };
                    Self::rgb(
                        component(i + offset)?,
                        component(i + offset + 1)?,
                        component(i + offset + 2)?,
                    )
                };
                if code == 38 {
                    self.foreground = color;
                } else {
                    self.background = color;
                }
            } else if code == 4 && end == i + 2 {
                self.attributes &= !(UNDERLINE | DOUBLE_UNDERLINE);
                self.attributes |= match value(i + 1)? {
                    0 => 0,
                    1 => UNDERLINE,
                    2 => DOUBLE_UNDERLINE,
                    _ => return None,
                };
            } else {
                if end != i + 1 {
                    return None;
                }
                match code {
                    0 => self = Self::default(),
                    1 => self.attributes |= BOLD,
                    2 => self.attributes |= FAINT,
                    3 => self.attributes |= ITALIC,
                    4 => {
                        self.attributes &= !DOUBLE_UNDERLINE;
                        self.attributes |= UNDERLINE;
                    }
                    7 => self.attributes |= INVERSE,
                    8 => self.attributes |= HIDDEN,
                    9 => self.attributes |= STRIKE,
                    21 => {
                        self.attributes &= !UNDERLINE;
                        self.attributes |= DOUBLE_UNDERLINE;
                    }
                    22 => self.attributes &= !(BOLD | FAINT),
                    23 => self.attributes &= !ITALIC,
                    24 => self.attributes &= !(UNDERLINE | DOUBLE_UNDERLINE),
                    27 => self.attributes &= !INVERSE,
                    28 => self.attributes &= !HIDDEN,
                    29 => self.attributes &= !STRIKE,
                    30..=37 => self.foreground = Self::indexed((code - 30) as u8),
                    39 => self.foreground = 0,
                    40..=47 => self.background = Self::indexed((code - 40) as u8),
                    49 => self.background = 0,
                    90..=97 => self.foreground = Self::indexed((code - 90 + 8) as u8),
                    100..=107 => self.background = Self::indexed((code - 100 + 8) as u8),
                    _ => return None,
                }
            }
            i = end;
        }
        Some(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Styles {
    entries: Vec<Option<Style>>,
}
impl Default for Styles {
    fn default() -> Self {
        Self {
            entries: vec![Some(Style::default())],
        }
    }
}
impl Styles {
    pub fn get(&self, id: u16) -> Option<Style> {
        self.entries.get(usize::from(id)).copied().flatten()
    }
    fn intern(&mut self, value: Style) -> Option<u16> {
        if let Some(id) = self.entries.iter().position(|s| *s == Some(value)) {
            return Some(id as u16);
        }
        if let Some(id) = self.entries.iter().position(Option::is_none) {
            self.entries[id] = Some(value);
            return Some(id as u16);
        }
        if self.entries.len() == MAX_STYLES {
            return None;
        }
        if self.entries.len() == self.entries.capacity() {
            let target = (self.entries.capacity() * 2).min(MAX_STYLES);
            self.entries.reserve_exact(target - self.entries.len());
        }
        let id = self.entries.len() as u16;
        self.entries.push(Some(value));
        Some(id)
    }
    pub fn heap_bytes(&self) -> usize {
        self.entries.capacity() * std::mem::size_of::<Option<Style>>()
    }
    pub fn palette(&self) -> impl ExactSizeIterator<Item = Style> + '_ {
        self.entries.iter().map(|s| s.unwrap_or_default())
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
impl Terminal {
    /// IDs are engine-local and may be recycled after no state references them.
    /// Use owned snapshots across mutation or when crossing the native boundary.
    pub fn style(&self, id: u16) -> Option<Style> {
        self.styles.get(id)
    }
    pub fn current_style(&self) -> Style {
        self.style(self.active.style).unwrap()
    }
    pub fn retained_style_count(&self) -> usize {
        self.styles.entries.iter().filter(|s| s.is_some()).count()
    }
    fn intern_style(&mut self, value: Style, keep: Option<u16>) -> Option<u16> {
        if let Some(id) = self.styles.intern(value) {
            return Some(id);
        }
        // Bounded mark/sweep only under allocation pressure. This includes hidden
        // primary/history, current and saved rendition, and the pending SGR pair.
        let mut used = [false; MAX_STYLES];
        used[0] = true;
        if let Some(id) = keep {
            used[usize::from(id)] = true;
        }
        for screen in std::iter::once(&self.active).chain(self.saved_primary.iter()) {
            for id in screen.style_roots() {
                used[usize::from(id)] = true;
            }
            for cell in screen
                .rows
                .iter()
                .chain(&screen.history)
                .flat_map(|r| &r.cells)
            {
                used[usize::from(cell.style_id())] = true;
            }
        }
        for (i, entry) in self.styles.entries.iter_mut().enumerate() {
            if !used[i] {
                *entry = None;
            }
        }
        self.styles.intern(value)
    }
    pub(crate) fn set_rendition(&mut self, value: Style) -> bool {
        let Some(style) = self.intern_style(value, None) else {
            return false;
        };
        let Some(erase) = self.intern_style(value.erased(), Some(style)) else {
            return false;
        };
        self.active.style = style;
        self.active.erase_style = erase;
        true
    }
}
