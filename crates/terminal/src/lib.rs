//! Owned terminal-state research slice. No I/O, async runtime or dependencies.
//! Bounded primary history, alternate screen and grapheme-preserving reflow.
mod decoder;
mod parser;
mod screen;
mod tables;
pub mod unicode;

use decoder::Decoder;
use parser::{Action, Parser};
use screen::Screen;
use std::collections::VecDeque;
use unicode::GraphemeBreak;
pub use unicode::WidthPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub columns: usize,
    pub lines: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_cells: usize,
    pub cluster_scalars: usize,
    pub history_rows: usize,
    pub history_cells: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_cells: 65_536,
            cluster_scalars: 64,
            history_rows: 1000,
            history_cells: 65_536,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ascii_cells_keep_text_inline() {
        let mut terminal = Terminal::new(
            Size {
                columns: 80,
                lines: 1,
            },
            Limits::default(),
            WidthPolicy::default(),
        )
        .unwrap();
        terminal.feed(&[b'x'; 80]);
        for cell in terminal.screen()[0].cells() {
            let Cell::Lead { cluster, width: 1 } = cell else {
                panic!("expected ASCII lead")
            };
            assert_eq!(cluster.extra.capacity(), 0);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidGeometry,
    InvalidLimits,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
    pub wrap_pending: bool,
}

/// Scalars live only in the lead. Single-scalar text never allocates a buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cluster {
    first: char,
    extra: Vec<char>,
}

impl Cluster {
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        std::iter::once(self.first).chain(self.extra.iter().copied())
    }
    pub fn scalar_count(&self) -> usize {
        1 + self.extra.len()
    }
    fn last(&self) -> char {
        self.extra.last().copied().unwrap_or(self.first)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Cell {
    #[default]
    Empty,
    Lead {
        cluster: Cluster,
        width: u8,
    },
    Continuation,
    WrapPadding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    cells: Vec<Cell>,
    soft_wrapped: bool,
}

impl Row {
    fn blank(columns: usize) -> Self {
        Self {
            cells: vec![Cell::Empty; columns],
            soft_wrapped: false,
        }
    }
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
    pub fn soft_wrapped(&self) -> bool {
        self.soft_wrapped
    }
}

/// Losses summed across both buffers. Primary counts use the reflowed width;
/// alternate counts use the old physical grid. History eviction is separate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResizeOutcome {
    /// Oldest primary rows dropped to satisfy the new history budget.
    pub history_evicted: usize,
    /// Rows cropped below the retained primary viewport or alternate grid.
    pub cropped_rows: usize,
    /// Cropped occupied cells, counting both wide halves and printed spaces,
    /// excluding empty cells and structural wrap padding.
    pub cropped_cells: usize,
}

/// Per-call flags aggregate diagnostics without allocating an event queue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeedOutcome {
    pub changed: bool,
    pub unsupported: bool,
    pub parser_limit: bool,
    pub cluster_limit: bool,
    pub orphan_mark: bool,
    pub scrolled_without_history: bool,
    pub history_evicted: bool,
}

impl FeedOutcome {
    pub fn merge(&mut self, other: Self) {
        self.changed |= other.changed;
        self.unsupported |= other.unsupported;
        self.parser_limit |= other.parser_limit;
        self.cluster_limit |= other.cluster_limit;
        self.orphan_mark |= other.orphan_mark;
        self.scrolled_without_history |= other.scrolled_without_history;
        self.history_evicted |= other.history_evicted;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    size: Size,
    limits: Limits,
    policy: WidthPolicy,
    active: Screen,
    saved_primary: Option<Screen>,
    decoder: Decoder,
    parser: Parser,
    segmenter: GraphemeBreak,
    last_lead: Option<(usize, usize)>,
}

impl Terminal {
    pub fn new(size: Size, limits: Limits, policy: WidthPolicy) -> Result<Self, Error> {
        if !(1..=64).contains(&limits.cluster_scalars)
            || !(1..=65_536).contains(&limits.max_cells)
            || limits.history_rows > 65_536
            || limits.history_cells > 65_536
        {
            return Err(Error::InvalidLimits);
        }
        if size.columns < 2
            || size.lines == 0
            || size
                .columns
                .checked_mul(size.lines)
                .is_none_or(|n| n > limits.max_cells)
        {
            return Err(Error::InvalidGeometry);
        }
        Ok(Self {
            size,
            limits,
            policy,
            active: Screen::new(size),
            saved_primary: None,
            decoder: Decoder::default(),
            parser: Parser::Ground,
            segmenter: GraphemeBreak::default(),
            last_lead: None,
        })
    }

    pub fn size(&self) -> Size {
        self.size
    }
    pub fn screen(&self) -> &[Row] {
        &self.active.rows
    }
    pub fn cursor(&self) -> Cursor {
        self.active.cursor
    }

    /// Active history is empty in the alternate screen. Primary history stays saved.
    pub fn history(&self) -> &VecDeque<Row> {
        &self.active.history
    }
    pub fn is_alternate(&self) -> bool {
        self.saved_primary.is_some()
    }

    fn history_capacity(&self, columns: usize) -> usize {
        self.limits
            .history_rows
            .min(self.limits.history_cells / columns)
    }

    fn alternate(&mut self, enable: bool, out: &mut FeedOutcome) {
        self.end_cluster();
        if enable && self.saved_primary.is_none() {
            self.saved_primary = Some(std::mem::replace(&mut self.active, Screen::new(self.size)));
            out.changed = true;
        } else if !enable && let Some(primary) = self.saved_primary.take() {
            self.active = primary;
            out.changed = true;
        }
    }

    /// Resize both screens; invalid geometry leaves the complete terminal unchanged.
    /// Every supported geometry succeeds. Primary reflow keeps the cursor visible,
    /// retaining preceding rows within history limits and cropping later rows only
    /// when necessary. Alternate state crops/pads. Losses from both buffers are
    /// reported. Geometry changes close the current extension target but preserve
    /// an incomplete UTF-8/escape sequence. Allocation failure is not recovered.
    pub fn resize(&mut self, size: Size) -> Result<ResizeOutcome, Error> {
        if size.columns < 2
            || size.lines == 0
            || size
                .columns
                .checked_mul(size.lines)
                .is_none_or(|n| n > self.limits.max_cells)
        {
            return Err(Error::InvalidGeometry);
        }
        if size == self.size {
            return Ok(ResizeOutcome::default());
        }
        let (active, mut out) = if self.saved_primary.is_some() {
            self.active.crop(size)
        } else {
            self.active.reflow(size, self.limits)
        };
        let saved = if let Some(primary) = self.saved_primary.as_ref() {
            let (screen, more) = primary.reflow(size, self.limits);
            out.history_evicted += more.history_evicted;
            out.cropped_rows += more.cropped_rows;
            out.cropped_cells += more.cropped_cells;
            Some(screen)
        } else {
            None
        };
        self.active = active;
        self.saved_primary = saved;
        self.size = size;
        self.end_cluster();
        debug_assert!(self.invariants_hold());
        Ok(out)
    }

    pub fn feed(&mut self, bytes: &[u8]) -> FeedOutcome {
        let mut result = FeedOutcome::default();
        for &byte in bytes {
            if self.parser != Parser::Ground {
                let action = self.parser.push(byte);
                self.action(action, &mut result);
            } else {
                for c in self.decoder.push(byte).into_iter().flatten() {
                    if c.is_ascii_control() || c == '\u{7f}' {
                        self.end_cluster();
                        let action = self.parser.push(c as u8);
                        self.action(action, &mut result);
                    } else if c.is_control() {
                        self.end_cluster();
                        result.unsupported = true;
                    } else {
                        self.print(c, &mut result);
                    }
                }
            }
        }
        debug_assert!(self.invariants_hold());
        result
    }

    /// Explicit end of input; a feed boundary alone never flushes a partial scalar.
    pub fn finish(&mut self) -> FeedOutcome {
        let mut result = FeedOutcome::default();
        if let Some(c) = self.decoder.finish() {
            self.print(c, &mut result);
        }
        if self.parser != Parser::Ground {
            result.unsupported = true;
        }
        self.parser = Parser::Ground;
        self.end_cluster();
        result
    }

    fn end_cluster(&mut self) {
        self.last_lead = None;
        self.segmenter = GraphemeBreak::default();
    }

    fn action(&mut self, action: Action, out: &mut FeedOutcome) {
        match action {
            Action::Alternate(enable) => self.alternate(enable, out),
            Action::None => {}
            Action::Unsupported => out.unsupported = true,
            Action::Limit => out.parser_limit = true,
            Action::Control(b) => {
                self.end_cluster();
                match b {
                    b'\r' => {
                        self.active.cursor.column = 0;
                        self.active.cursor.wrap_pending = false;
                        out.changed = true;
                    }
                    b'\n' => {
                        self.active.rows[self.active.cursor.row].soft_wrapped = false;
                        self.down(out);
                        self.active.cursor.wrap_pending = false;
                    }
                    8 => {
                        self.active.cursor.column = self.active.cursor.column.saturating_sub(1);
                        self.active.cursor.wrap_pending = false;
                        out.changed = true;
                    }
                    0 => {}
                    _ => out.unsupported = true,
                }
            }
            Action::Left(n) => {
                self.end_cluster();
                self.active.cursor.column = self.active.cursor.column.saturating_sub(n);
                self.active.cursor.wrap_pending = false;
                out.changed = true;
            }
            Action::Erase(n) => {
                self.end_cluster();
                self.erase(
                    self.active.cursor.row,
                    self.active.cursor.column,
                    (self.active.cursor.column + n).min(self.size.columns),
                );
                out.changed = true;
            }
            Action::EraseLine => {
                self.end_cluster();
                self.erase(
                    self.active.cursor.row,
                    self.active.cursor.column,
                    self.size.columns,
                );
                self.active.rows[self.active.cursor.row].soft_wrapped = false;
                out.changed = true;
            }
        }
    }

    fn erase(&mut self, row: usize, start: usize, end: usize) {
        let cells = &mut self.active.rows[row].cells;
        let start = if matches!(cells[start], Cell::Continuation) {
            start - 1
        } else {
            start
        };
        let end = if end < cells.len() && matches!(cells[end], Cell::Continuation) {
            end + 1
        } else {
            end
        };
        cells[start..end].fill(Cell::Empty);
    }

    fn down(&mut self, out: &mut FeedOutcome) {
        if self.active.cursor.row + 1 < self.size.lines {
            self.active.cursor.row += 1;
        } else {
            self.active.rows.rotate_left(1);
            let old = std::mem::replace(
                self.active.rows.last_mut().unwrap(),
                Row::blank(self.size.columns),
            );
            let cap = self.history_capacity(self.size.columns);
            if self.saved_primary.is_none() && cap > 0 {
                if self.active.history.len() == cap {
                    self.active.history.pop_front();
                    out.history_evicted = true;
                }
                self.active.history.push_back(old);
            } else {
                out.scrolled_without_history = true;
            }
        }
        out.changed = true;
    }

    fn wrap(&mut self, out: &mut FeedOutcome) {
        self.active.rows[self.active.cursor.row].soft_wrapped = true;
        self.down(out);
        self.active.cursor.column = 0;
        self.active.cursor.wrap_pending = false;
    }

    fn set_after(&mut self, row: usize, col: usize, width: u8) {
        let next = col + usize::from(width);
        self.active.cursor = Cursor {
            row,
            column: next.min(self.size.columns - 1),
            wrap_pending: next == self.size.columns,
        };
    }

    fn print(&mut self, c: char, out: &mut FeedOutcome) {
        let boundary = self.segmenter.push(c);
        if !boundary && let Some((row, col)) = self.last_lead {
            let (old_width, new_width) = match &mut self.active.rows[row].cells[col] {
                Cell::Lead { cluster, width } => {
                    if cluster.scalar_count() >= self.limits.cluster_scalars {
                        out.cluster_limit = true;
                        return; // Segmentation still advances: retain a bounded prefix until next boundary.
                    }
                    let new = self.policy.extend(*width, cluster.last(), c).max(1);
                    cluster.extra.push(c);
                    (*width, new)
                }
                _ => unreachable!("extension target is always a lead"),
            };
            if old_width != new_width {
                let Cell::Lead { cluster, .. } =
                    std::mem::take(&mut self.active.rows[row].cells[col])
                else {
                    unreachable!()
                };
                if old_width == 2 {
                    self.active.rows[row].cells[col + 1] = Cell::Empty;
                }
                self.active.cursor = Cursor {
                    row,
                    column: col,
                    wrap_pending: false,
                };
                self.place(cluster, new_width, out);
            }
            out.changed = true;
            return;
        }
        self.last_lead = None;
        let width = self.policy.scalar(c);
        if width == 0 {
            out.orphan_mark = true; // Explicit slice policy: discard unattached zero-width text.
            return;
        }
        self.place(
            Cluster {
                first: c,
                extra: Vec::new(),
            },
            width,
            out,
        );
    }

    fn place(&mut self, cluster: Cluster, width: u8, out: &mut FeedOutcome) {
        if self.active.cursor.wrap_pending {
            self.wrap(out);
        }
        if width == 2 && self.active.cursor.column == self.size.columns - 1 {
            self.erase(
                self.active.cursor.row,
                self.active.cursor.column,
                self.size.columns,
            );
            self.active.rows[self.active.cursor.row].cells[self.active.cursor.column] =
                Cell::WrapPadding;
            self.wrap(out);
        }
        let (row, col) = (self.active.cursor.row, self.active.cursor.column);
        self.erase(row, col, col + usize::from(width));
        self.active.rows[row].cells[col] = Cell::Lead { cluster, width };
        if width == 2 {
            self.active.rows[row].cells[col + 1] = Cell::Continuation;
        }
        self.last_lead = Some((row, col));
        self.set_after(row, col, width);
        out.changed = true;
    }

    /// Check structural invariants without allocating; useful to replay/fuzz callers.
    pub fn invariants_hold(&self) -> bool {
        if !self.active.valid(self.size, self.limits)
            || (self.saved_primary.is_some() && !self.active.history.is_empty())
            || self
                .saved_primary
                .as_ref()
                .is_some_and(|p| !p.valid(self.size, self.limits))
        {
            return false;
        }
        self.last_lead.is_none_or(|(r, c)| {
            matches!(
                self.active.rows.get(r).and_then(|row| row.cells.get(c)),
                Some(Cell::Lead { .. })
            )
        })
    }
}
