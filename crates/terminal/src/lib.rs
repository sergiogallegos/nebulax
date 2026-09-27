//! Owned terminal-state research slice. No I/O, async runtime or dependencies.
//! Bounded primary history, alternate screen and grapheme-preserving reflow.
mod cell;
pub use cell::{Cell, CellView, Cluster};
mod decoder;
pub mod input;
pub mod output;
pub mod parser;
mod screen;
mod semantic;
pub mod snapshot;
pub mod storage;
pub mod style;
mod tables;
pub mod unicode;

use decoder::Decoder;
use output::Output;
pub use output::{OutputEvent, TitleTarget};
use parser::Parser;
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
            let CellView::Lead { cluster, width: 1 } = cell.view() else {
                panic!("expected ASCII lead")
            };
            assert_eq!(cluster.scalar_count(), 1);
            assert_eq!(cell.heap_bytes(), 0);
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    cells: Vec<Cell>,
    soft_wrapped: bool,
}

impl Row {
    fn blank(columns: usize) -> Self {
        Self {
            cells: vec![Cell::EMPTY; columns],
            soft_wrapped: false,
        }
    }
    fn clear(&mut self, style: u16) {
        self.cells.fill(Cell::EMPTY.with_style(style));
        self.soft_wrapped = false;
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

/// Per-call progress and diagnostics. Resume at `bytes[consumed..]` after draining
/// output if `output_blocked` is true. A completed event is never silently lost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeedOutcome {
    pub consumed: usize,
    pub output_blocked: bool,
    pub changed: bool,
    pub unsupported: bool,
    pub parser_limit: bool,
    pub cluster_limit: bool,
    pub style_limit: bool,
    pub orphan_mark: bool,
    pub scrolled_without_history: bool,
    pub history_evicted: bool,
}

impl FeedOutcome {
    pub fn merge(&mut self, other: Self) {
        self.consumed += other.consumed;
        self.output_blocked = other.output_blocked;
        self.changed |= other.changed;
        self.unsupported |= other.unsupported;
        self.parser_limit |= other.parser_limit;
        self.cluster_limit |= other.cluster_limit;
        self.style_limit |= other.style_limit;
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
    output: Output,
    application_cursor: bool,
    styles: style::Styles,
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
            parser: Parser::default(),
            output: Output::default(),
            application_cursor: false,
            styles: style::Styles::default(),
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

    /// Consume a prefix, stopping immediately if a completed output event waits
    /// for capacity. Drain `pop_output`, then resume the unconsumed suffix.
    pub fn feed(&mut self, bytes: &[u8]) -> FeedOutcome {
        let mut result = FeedOutcome::default();
        if !self.output.blocked() {
            for &byte in bytes {
                if let Some(event) = self.parser.push(byte) {
                    self.dispatch(event, &mut result);
                }
                result.consumed += 1;
                if self.output.blocked() {
                    break;
                }
            }
        }
        result.output_blocked = self.output.blocked();
        debug_assert!(self.invariants_hold());
        result
    }

    /// Explicit EOF. If output is blocked, drain it and retry: no decoder/parser
    /// state is discarded until this completes without `output_blocked`.
    pub fn finish(&mut self) -> FeedOutcome {
        let mut result = FeedOutcome::default();
        if self.output.blocked() {
            result.output_blocked = true;
            return result;
        }
        self.flush_decoder(&mut result);
        if !self.parser.is_ground() {
            result.unsupported = true;
        }
        self.parser.reset();
        self.end_cluster();
        result
    }

    /// Removes the oldest inert event and promotes a waiting event if it fits.
    /// The caller owns policy checks and PTY writes, including partial writes.
    pub fn pop_output(&mut self) -> Option<OutputEvent> {
        self.output.pop()
    }
    /// Includes the one completed event waiting outside a full queue.
    pub fn pending_output_len(&self) -> usize {
        self.output.len()
    }
    pub fn pending_output_bytes(&self) -> usize {
        self.output.bytes()
    }

    fn flush_decoder(&mut self, out: &mut FeedOutcome) {
        if let Some(c) = self.decoder.finish() {
            self.print(c, out);
        }
    }

    fn end_cluster(&mut self) {
        self.last_lead = None;
        self.segmenter = GraphemeBreak::default();
    }

    fn erase(&mut self, row: usize, start: usize, end: usize) {
        self.active.erase(row, start, end);
    }

    fn down(&mut self, out: &mut FeedOutcome) {
        let cap = if self.is_alternate() {
            0
        } else {
            self.history_capacity(self.size.columns)
        };
        self.active.down(cap, out);
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
            let cell = &mut self.active.rows[row].cells[col];
            let CellView::Lead {
                cluster,
                width: old_width,
            } = cell.view()
            else {
                unreachable!("extension target is a lead")
            };
            if cluster.scalar_count() >= self.limits.cluster_scalars {
                out.cluster_limit = true;
                return; // Segmentation still advances; retain a bounded prefix.
            }
            let new_width = self.policy.extend(old_width, cluster.last(), c).max(1);
            cell.push(c);
            if old_width != new_width {
                let mut cell = std::mem::take(cell);
                cell.set_width(new_width);
                if old_width == 2 {
                    self.active.rows[row].cells[col + 1] =
                        Cell::EMPTY.with_style(self.active.erase_style);
                }
                self.active.cursor = Cursor {
                    row,
                    column: col,
                    wrap_pending: false,
                };
                self.place(cell, new_width, out);
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
            Cell::lead(c, width).with_style(self.active.style),
            width,
            out,
        );
    }

    fn place(&mut self, cell: Cell, width: u8, out: &mut FeedOutcome) {
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
                Cell::WRAP_PADDING.with_style(cell.style_id());
            self.wrap(out);
        }
        let (row, col) = (self.active.cursor.row, self.active.cursor.column);
        self.erase(row, col, col + usize::from(width));
        let style = cell.style_id();
        self.active.rows[row].cells[col] = cell;
        if width == 2 {
            self.active.rows[row].cells[col + 1] = Cell::CONTINUATION.with_style(style);
        }
        self.last_lead = Some((row, col));
        self.set_after(row, col, width);
        out.changed = true;
    }

    /// Check structural invariants without allocating; useful to replay/fuzz callers.
    pub fn invariants_hold(&self) -> bool {
        if !self.output.valid()
            || !self.active.valid(self.size, self.limits)
            || (self.saved_primary.is_some() && !self.active.history.is_empty())
            || self
                .saved_primary
                .as_ref()
                .is_some_and(|p| !p.valid(self.size, self.limits))
        {
            return false;
        }
        for screen in std::iter::once(&self.active).chain(self.saved_primary.iter()) {
            if screen
                .style_roots()
                .iter()
                .any(|id| self.style(*id).is_none())
                || screen
                    .rows
                    .iter()
                    .chain(&screen.history)
                    .flat_map(|r| &r.cells)
                    .any(|c| self.style(c.style_id()).is_none())
            {
                return false;
            }
        }
        self.last_lead.is_none_or(|(r, c)| {
            matches!(
                self.active
                    .rows
                    .get(r)
                    .and_then(|row| row.cells.get(c))
                    .map(Cell::view),
                Some(CellView::Lead { .. })
            )
        })
    }
}
