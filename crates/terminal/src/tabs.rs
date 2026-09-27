//! Terminal-wide tab stops, retained across screen switches and shrink/grow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tabs {
    words: Vec<u64>,
    default_new: bool,
}
impl Tabs {
    pub fn new(columns: usize) -> Self {
        let mut result = Self {
            words: Vec::new(),
            default_new: true,
        };
        result.grow(columns);
        result.set(0, false);
        result
    }
    pub fn grow(&mut self, columns: usize) {
        let count = columns.div_ceil(64);
        if count > self.words.len() {
            self.words.reserve_exact(count - self.words.len());
            self.words.resize(
                count,
                if self.default_new {
                    0x0101_0101_0101_0101
                } else {
                    0
                },
            );
        }
    }
    pub fn set(&mut self, column: usize, enabled: bool) {
        let mask = 1 << (column % 64);
        if enabled {
            self.words[column / 64] |= mask;
        } else {
            self.words[column / 64] &= !mask;
        }
    }
    pub fn clear(&mut self) {
        self.words.fill(0);
        self.default_new = false;
    }
    fn contains(&self, column: usize) -> bool {
        self.words[column / 64] & (1 << (column % 64)) != 0
    }
    pub fn destination(&self, column: usize, columns: usize, count: usize, forward: bool) -> usize {
        // Scan the visible width once, even for a maximal numeric parameter.
        if forward {
            (column + 1..columns)
                .filter(|&c| self.contains(c))
                .nth(count - 1)
                .unwrap_or(columns - 1)
        } else {
            (0..column)
                .rev()
                .filter(|&c| self.contains(c))
                .nth(count - 1)
                .unwrap_or(0)
        }
    }
    pub fn heap_bytes(&self) -> usize {
        self.words.capacity() * 8
    }
    pub fn valid(&self, columns: usize) -> bool {
        self.words.len() >= columns.div_ceil(64) && self.words.capacity() <= 1024
    }
}
