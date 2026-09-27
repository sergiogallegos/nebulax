//! Owned screen storage and bounded resize. Original implementation.
use crate::{Cell, CellView, Cursor, FeedOutcome, Limits, ResizeOutcome, Row, Size};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SavedCursor {
    cursor: Cursor,
    origin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Screen {
    pub rows: Vec<Row>,
    pub history: VecDeque<Row>,
    pub cursor: Cursor,
    pub origin: bool,
    pub top: usize,
    pub bottom: usize,
    saved_cursor: SavedCursor,
}

impl Screen {
    pub fn new(size: Size) -> Self {
        Self {
            rows: (0..size.lines).map(|_| Row::blank(size.columns)).collect(),
            history: VecDeque::new(),
            cursor: Cursor::default(),
            origin: false,
            top: 0,
            bottom: size.lines - 1,
            saved_cursor: SavedCursor::default(),
        }
    }

    pub fn position(&mut self, row: usize, column: usize) {
        let top = if self.origin { self.top } else { 0 };
        let bottom = if self.origin {
            self.bottom
        } else {
            self.rows.len() - 1
        };
        self.cursor = Cursor {
            row: top.saturating_add(row).min(bottom),
            column: column.min(self.rows[0].cells.len() - 1),
            wrap_pending: false,
        };
    }

    pub fn relative(&mut self, vertical: isize, horizontal: isize) {
        let low = if self.cursor.row >= self.top {
            self.top
        } else {
            0
        };
        let high = if self.cursor.row <= self.bottom {
            self.bottom
        } else {
            self.rows.len() - 1
        };
        self.cursor.row = self
            .cursor
            .row
            .saturating_add_signed(vertical)
            .clamp(low, high);
        self.cursor.column = self
            .cursor
            .column
            .saturating_add_signed(horizontal)
            .min(self.rows[0].cells.len() - 1);
        self.cursor.wrap_pending = false;
    }

    pub fn set_origin(&mut self, enabled: bool) {
        self.origin = enabled;
        self.position(0, 0);
    }

    /// Wire coordinates are one-based; zero/omitted bounds mean full extent.
    /// Invalid regions leave margins and cursor untouched.
    pub fn set_margins(&mut self, top: usize, bottom: usize) -> bool {
        let top = top.max(1) - 1;
        let bottom = if bottom == 0 { self.rows.len() } else { bottom } - 1;
        if top >= bottom || bottom >= self.rows.len() {
            return false;
        }
        self.top = top;
        self.bottom = bottom;
        self.position(0, 0);
        true
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor = SavedCursor {
            cursor: self.cursor,
            origin: self.origin,
        };
    }

    pub fn restore_cursor(&mut self) {
        self.origin = self.saved_cursor.origin;
        self.cursor = self.saved_cursor.cursor;
        if self.origin {
            self.cursor.row = self.cursor.row.clamp(self.top, self.bottom);
        }
    }

    // Resize resets margins on both buffers, retains origin, and clamps the
    // saved physical position. Only the active cursor participates in reflow.
    fn resized_state(&self, result: &mut Self, size: Size) {
        result.origin = self.origin;
        result.saved_cursor = self.saved_cursor;
        let saved = &mut result.saved_cursor.cursor;
        saved.row = saved.row.min(size.lines - 1);
        saved.column = saved.column.min(size.columns - 1);
        saved.wrap_pending &= self.saved_cursor.cursor.column + 1 == size.columns;
    }

    pub fn erase(&mut self, row: usize, start: usize, end: usize) {
        let cells = &mut self.rows[row].cells;
        let start = if matches!(cells[start].view(), CellView::Continuation) {
            start - 1
        } else {
            start
        };
        let end = if end < cells.len() && matches!(cells[end].view(), CellView::Continuation) {
            end + 1
        } else {
            end
        };
        cells[start..end].fill(Cell::EMPTY);
    }

    fn break_wrap(&mut self, row: usize) {
        self.rows[row].soft_wrapped = false;
        if self.rows[row].cells.last() == Some(&Cell::WRAP_PADDING) {
            *self.rows[row].cells.last_mut().unwrap() = Cell::EMPTY;
        }
    }

    fn detach_region_start(&mut self) {
        if self.top > 0 {
            self.break_wrap(self.top - 1);
        } else if let Some(row) = self.history.back_mut() {
            row.soft_wrapped = false;
            if row.cells.last() == Some(&Cell::WRAP_PADDING) {
                *row.cells.last_mut().unwrap() = Cell::EMPTY;
            }
        }
    }

    pub fn down(&mut self, history_capacity: usize, out: &mut FeedOutcome) {
        if self.cursor.row == self.bottom {
            if self.top != 0 || self.bottom + 1 != self.rows.len() {
                self.detach_region_start();
            }
            self.rows[self.top..=self.bottom].rotate_left(1);
            if self.top == 0 && self.bottom + 1 == self.rows.len() && history_capacity > 0 {
                let blank = if self.history.len() == history_capacity {
                    let mut row = self.history.pop_front().unwrap();
                    row.clear();
                    out.history_evicted = true;
                    row
                } else {
                    Row::blank(self.rows[0].cells.len())
                };
                let old = std::mem::replace(&mut self.rows[self.bottom], blank);
                self.history.push_back(old);
            } else {
                self.rows[self.bottom].clear();
                out.scrolled_without_history = true;
            }
        } else if self.cursor.row + 1 < self.rows.len() {
            self.cursor.row += 1;
        } else {
            self.break_wrap(self.cursor.row);
        }
        self.cursor.wrap_pending = false;
        out.changed = true;
    }

    pub fn reverse_index(&mut self, out: &mut FeedOutcome) {
        if self.cursor.row == self.top {
            self.detach_region_start();
            self.rows[self.top..=self.bottom].rotate_right(1);
            self.rows[self.top].clear();
            self.break_wrap(self.bottom);
            out.scrolled_without_history = true;
        } else {
            self.cursor.row = self.cursor.row.saturating_sub(1);
        }
        self.cursor.wrap_pending = false;
        out.changed = true;
    }

    pub fn valid(&self, size: Size, limits: Limits) -> bool {
        if self.rows.len() != size.lines
            || self.top > self.bottom
            || self.bottom >= size.lines
            || (self.origin && !(self.top..=self.bottom).contains(&self.cursor.row))
            || self.saved_cursor.cursor.row >= size.lines
            || self.saved_cursor.cursor.column >= size.columns
            || (self.saved_cursor.cursor.wrap_pending
                && self.saved_cursor.cursor.column + 1 != size.columns)
            || self.cursor.row >= size.lines
            || self.cursor.column >= size.columns
            || (self.cursor.wrap_pending && self.cursor.column + 1 != size.columns)
            || self.history.len() > limits.history_rows.min(limits.history_cells / size.columns)
        {
            return false;
        }
        for row in self.history.iter().chain(&self.rows) {
            if row.cells.len() != size.columns {
                return false;
            }
            for (i, cell) in row.cells.iter().enumerate() {
                if !cell.valid() {
                    return false;
                }
                match cell.view() {
                    CellView::Lead { cluster, width } => {
                        if !matches!(width, 1 | 2)
                            || cluster.scalar_count() > limits.cluster_scalars
                        {
                            return false;
                        }
                        if width == 2 && row.cells.get(i + 1) != Some(&Cell::CONTINUATION) {
                            return false;
                        }
                    }
                    CellView::Continuation => {
                        if i == 0
                            || !matches!(row.cells[i - 1].view(), CellView::Lead { width: 2, .. })
                        {
                            return false;
                        }
                    }
                    CellView::WrapPadding => {
                        if i + 1 != size.columns {
                            return false;
                        }
                    }
                    CellView::Empty => {}
                }
            }
        }
        true
    }

    /// Alternate screen preserves physical coordinates; clipped wide halves
    /// are removed together. No reflow or history is used for TUI state.
    pub fn crop(&self, size: Size) -> (Self, ResizeOutcome) {
        let mut result = Self::new(size);
        let mut outcome = ResizeOutcome::default();
        for (y, old) in self.rows.iter().enumerate() {
            if y >= size.lines {
                outcome.cropped_rows += 1;
                outcome.cropped_cells += old
                    .cells
                    .iter()
                    .filter(|c| !matches!(c.view(), CellView::Empty | CellView::WrapPadding))
                    .count();
                continue;
            }
            for (x, c) in old.cells.iter().enumerate() {
                if x >= size.columns {
                    outcome.cropped_cells +=
                        usize::from(!matches!(c.view(), CellView::Empty | CellView::WrapPadding));
                    continue;
                }
                result.rows[y].cells[x] = match c.view() {
                    CellView::Lead { width: 2, .. } if x + 1 == size.columns => {
                        outcome.cropped_cells += 1;
                        Cell::EMPTY
                    }
                    CellView::WrapPadding => Cell::EMPTY,
                    _ => c.clone(),
                };
            }
            // Cropping changes adjacency; only unchanged-width rows retain wrap.
            result.rows[y].soft_wrapped = old.soft_wrapped && old.cells.len() == size.columns;
        }
        result.cursor = Cursor {
            row: self.cursor.row.min(size.lines - 1),
            column: self.cursor.column.min(size.columns - 1),
            wrap_pending: self.cursor.wrap_pending && self.cursor.column + 1 == size.columns,
        };
        if outcome.cropped_rows != 0 {
            result.rows.last_mut().unwrap().soft_wrapped = false;
        }
        self.resized_state(&mut result, size);
        (result, outcome)
    }

    /// Repack complete cluster owners, joining only rows marked soft-wrapped.
    /// Cursor is a logical cell offset, including the one-past-edge pending state.
    pub fn reflow(&self, size: Size, limits: Limits) -> (Self, ResizeOutcome) {
        let cap = limits.history_rows.min(limits.history_cells / size.columns);
        let mut sink = ReflowSink::new(size, cap);
        let cursor_row = self.history.len() + self.cursor.row;
        let significant = self
            .history
            .iter()
            .chain(&self.rows)
            .enumerate()
            .filter(|(_, row)| {
                row.soft_wrapped
                    || row
                        .cells
                        .iter()
                        .any(|c| !matches!(c.view(), CellView::Empty | CellView::WrapPadding))
            })
            .map(|(i, _)| i + 1)
            .max()
            .unwrap_or(0)
            .max(cursor_row + 1);
        let mut logical = Vec::new();
        let mut logical_width = 0;
        let mut anchor = None;
        for (y, row) in self
            .history
            .iter()
            .chain(&self.rows)
            .take(significant)
            .enumerate()
        {
            let cursor_end = self.cursor.column + usize::from(self.cursor.wrap_pending);
            if y == cursor_row {
                let offset = row.cells[..cursor_end]
                    .iter()
                    .filter(|c| !matches!(c.view(), CellView::WrapPadding))
                    .count();
                anchor = Some((logical_width + offset, self.cursor.wrap_pending));
            }
            let extent = if row.soft_wrapped {
                row.cells.len()
            } else {
                row.cells
                    .iter()
                    .rposition(|c| !matches!(c.view(), CellView::Empty | CellView::WrapPadding))
                    .map_or(0, |i| i + 1)
                    .max(if y == cursor_row { cursor_end } else { 0 })
            };
            for c in &row.cells[..extent] {
                match c.view() {
                    CellView::Empty => {
                        logical_width += 1;
                        logical.push(Cell::EMPTY);
                    }
                    CellView::Lead { width, .. } => {
                        logical_width += usize::from(width);
                        logical.push(c.clone());
                    }
                    CellView::Continuation | CellView::WrapPadding => {}
                }
            }
            if !row.soft_wrapped || y + 1 == significant {
                sink.line(logical.drain(..), anchor.take());
                logical_width = 0;
            }
        }
        let (mut result, outcome) = sink.finish();
        self.resized_state(&mut result, size);
        (result, outcome)
    }
}

/// Keep at most history-cap + viewport rows, even when old hard lines would
/// expand into billions of blank cells at a very wide new geometry.
struct ReflowSink {
    size: Size,
    capacity: usize,
    rows: VecDeque<Row>,
    base: usize,
    produced: usize,
    cursor: Option<Cursor>,
    outcome: ResizeOutcome,
}

impl ReflowSink {
    fn new(size: Size, history: usize) -> Self {
        Self {
            size,
            capacity: history + size.lines,
            rows: VecDeque::new(),
            base: 0,
            produced: 0,
            cursor: None,
            outcome: ResizeOutcome::default(),
        }
    }

    fn push(&mut self, row: Row) {
        if self
            .cursor
            .is_some_and(|c| self.produced >= c.row + self.size.lines)
        {
            self.outcome.cropped_rows += 1;
            self.outcome.cropped_cells += row
                .cells
                .iter()
                .filter(|c| !matches!(c.view(), CellView::Empty | CellView::WrapPadding))
                .count();
        } else {
            if self.rows.len() == self.capacity {
                self.rows.pop_front();
                self.base += 1;
                self.outcome.history_evicted += 1;
            }
            self.rows.push_back(row);
        }
        self.produced += 1;
    }

    fn line(&mut self, cells: impl Iterator<Item = Cell>, anchor: Option<(usize, bool)>) {
        let mut row = Row {
            cells: Vec::new(),
            soft_wrapped: false,
        };
        let mut x = 0;
        let mut consumed = 0;
        let mut mapped = false;
        for cell in cells {
            let width = match cell.view() {
                CellView::Lead { width, .. } => usize::from(width),
                _ => 1,
            };
            // A saved pending-wrap position has left affinity at an exact edge.
            if !mapped && anchor == Some((consumed, true)) && x == self.size.columns {
                self.cursor = Some(Cursor {
                    row: self.produced,
                    column: x - 1,
                    wrap_pending: true,
                });
                mapped = true;
            }
            if x + width > self.size.columns {
                if x < self.size.columns {
                    row.cells.push(Cell::WRAP_PADDING);
                }
                row.soft_wrapped = true;
                self.push(row);
                row = Row {
                    cells: Vec::new(),
                    soft_wrapped: false,
                };
                x = 0;
            }
            if !mapped
                && let Some((offset, _)) = anchor
                && (consumed..consumed + width).contains(&offset)
            {
                self.cursor = Some(Cursor {
                    row: self.produced,
                    column: x + offset - consumed,
                    wrap_pending: false,
                });
                mapped = true;
            }
            row.cells.push(cell);
            if width == 2 {
                row.cells.push(Cell::CONTINUATION);
            }
            x += width;
            consumed += width;
        }
        if let Some((offset, _)) = anchor
            && !mapped
        {
            debug_assert_eq!(offset, consumed);
            self.cursor = Some(Cursor {
                row: self.produced,
                column: x.min(self.size.columns - 1),
                wrap_pending: x == self.size.columns,
            });
        }
        self.push(row);
    }

    fn finish(mut self) -> (Screen, ResizeOutcome) {
        // The retained suffix ends here: it must not stay linked to discarded
        // content or carry padding whose wide owner was in that content.
        if self.outcome.cropped_rows != 0 {
            let last = self.rows.back_mut().unwrap();
            last.soft_wrapped = false;
            if last.cells.last() == Some(&Cell::WRAP_PADDING) {
                *last.cells.last_mut().unwrap() = Cell::EMPTY;
            }
        }
        let mut cursor = self
            .cursor
            .expect("cursor line always participates in reflow");
        cursor.row -= self.base;
        let start = self
            .rows
            .len()
            .saturating_sub(self.size.lines)
            .min(cursor.row);
        let history = self
            .rows
            .drain(..start)
            .map(|mut row| {
                row.cells.resize(self.size.columns, Cell::EMPTY);
                row
            })
            .collect();
        cursor.row -= start;
        let mut rows: Vec<_> = self
            .rows
            .into_iter()
            .take(self.size.lines)
            .map(|mut row| {
                row.cells.resize(self.size.columns, Cell::EMPTY);
                row
            })
            .collect();
        rows.resize_with(self.size.lines, || Row::blank(self.size.columns));
        (
            Screen {
                rows,
                history,
                cursor,
                origin: false,
                top: 0,
                bottom: self.size.lines - 1,
                saved_cursor: SavedCursor::default(),
            },
            self.outcome,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Terminal, WidthPolicy};

    #[test]
    fn hard_rows_below_cursor_are_cropped_without_reordering_into_history() {
        let mut t = Terminal::new(
            Size {
                columns: 4,
                lines: 4,
            },
            Limits::default(),
            WidthPolicy::default(),
        )
        .unwrap();
        t.feed(b"aa\r\n\r\nbb\r\ncc");
        // Set the cursor directly: cursor-up is deliberately outside the parser's
        // current subset, but resize must support this valid screen state.
        t.active.cursor = Cursor {
            row: 1,
            column: 0,
            wrap_pending: false,
        };
        let out = t
            .resize(Size {
                columns: 4,
                lines: 1,
            })
            .unwrap();
        assert_eq!(
            out,
            ResizeOutcome {
                history_evicted: 0,
                cropped_rows: 2,
                cropped_cells: 4
            }
        );
        assert_eq!(t.history().len(), 1);
        assert!(!t.history()[0].soft_wrapped());
        assert!(t.screen()[0].cells().iter().all(|c| *c == Cell::EMPTY));
        assert_eq!(t.cursor(), Cursor::default());
        t.resize(Size {
            columns: 8,
            lines: 4,
        })
        .unwrap();
        assert!(matches!(
            t.screen()[0].cells()[0].view(),
            CellView::Lead { .. }
        ));
        assert!(!t.screen()[0].soft_wrapped());
        assert!(
            t.screen()[1..]
                .iter()
                .flat_map(Row::cells)
                .all(|c| *c == Cell::EMPTY)
        );
        assert!(t.invariants_hold());
    }
}
