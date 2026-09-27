//! Owned screen storage and bounded resize. Original implementation.
use crate::{Cell, CellView, Cursor, FeedOutcome, Limits, ResizeOutcome, Row, Size};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SavedCursor {
    cursor: Cursor,
    origin: bool,
    autowrap: bool,
    style: u16,
    erase_style: u16,
}

impl Default for SavedCursor {
    fn default() -> Self {
        Self {
            cursor: Cursor::default(),
            origin: false,
            autowrap: true,
            style: 0,
            erase_style: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Screen {
    pub rows: Vec<Row>,
    pub history: VecDeque<Row>,
    pub cursor: Cursor,
    pub origin: bool,
    pub autowrap: bool,
    pub style: u16,
    pub erase_style: u16,
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
            autowrap: true,
            style: 0,
            erase_style: 0,
            top: 0,
            bottom: size.lines - 1,
            saved_cursor: SavedCursor::default(),
        }
    }

    pub fn style_roots(&self) -> [u16; 4] {
        [
            self.style,
            self.erase_style,
            self.saved_cursor.style,
            self.saved_cursor.erase_style,
        ]
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
            autowrap: self.autowrap,
            style: self.style,
            erase_style: self.erase_style,
        };
    }

    pub fn restore_cursor(&mut self) {
        self.origin = self.saved_cursor.origin;
        self.autowrap = self.saved_cursor.autowrap;
        self.style = self.saved_cursor.style;
        self.erase_style = self.saved_cursor.erase_style;
        self.cursor = self.saved_cursor.cursor;
        if self.origin {
            self.cursor.row = self.cursor.row.clamp(self.top, self.bottom);
        }
    }

    // Resize resets margins on both buffers, retains origin, and clamps the
    // saved physical position. Only the active cursor participates in reflow.
    fn resized_state(&self, result: &mut Self, size: Size) {
        result.origin = self.origin;
        result.autowrap = self.autowrap;
        result.cursor.wrap_pending &= self.autowrap;
        result.style = self.style;
        result.erase_style = self.erase_style;
        result.saved_cursor = self.saved_cursor;
        let saved = &mut result.saved_cursor.cursor;
        saved.row = saved.row.min(size.lines - 1);
        saved.column = saved.column.min(size.columns - 1);
        saved.wrap_pending &= self.saved_cursor.cursor.column + 1 == size.columns;
    }

    pub fn erase(&mut self, row: usize, start: usize, end: usize) -> (usize, usize) {
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
        cells[start..end].fill(Cell::EMPTY.with_style(self.erase_style));
        (start, end)
    }

    /// Shift physical columns without cloning cluster allocations.
    pub fn edit_characters(&mut self, count: usize, insert: bool) {
        let row = self.cursor.row;
        let start = self.cursor.column;
        let columns = self.rows[row].cells.len();
        let count = count.min(columns - start);
        self.break_wrap(row); // Structural padding must never move into a row.
        let starts_in_tail = matches!(self.rows[row].cells[start].view(), CellView::Continuation);
        if start == 0 || (start == 1 && starts_in_tail) {
            self.detach_before(row);
        }
        if insert {
            if starts_in_tail {
                self.erase(row, start, start + 1);
            }
            let cutoff = columns - count;
            if matches!(self.rows[row].cells[cutoff].view(), CellView::Continuation) {
                self.erase(row, cutoff, cutoff + 1);
            }
            self.rows[row].cells[start..].rotate_right(count);
            self.rows[row].cells[start..start + count]
                .fill(Cell::EMPTY.with_style(self.erase_style));
        } else {
            self.erase(row, start, start + count);
            self.rows[row].cells[start..].rotate_left(count);
            self.rows[row].cells[columns - count..].fill(Cell::EMPTY.with_style(self.erase_style));
        }
        self.cursor.wrap_pending = false;
    }

    /// Move whole rows only inside the cursor-to-bottom part of the region.
    pub fn edit_lines(&mut self, count: usize, insert: bool) -> bool {
        let row = self.cursor.row;
        if !(self.top..=self.bottom).contains(&row) {
            return false;
        }
        let count = count.min(self.bottom + 1 - row);
        self.detach_before(row);
        if insert {
            self.rows[row..=self.bottom].rotate_right(count);
            for y in row..row + count {
                self.rows[y].clear(self.erase_style);
            }
            self.break_wrap(self.bottom);
        } else {
            self.rows[row..=self.bottom].rotate_left(count);
            let blank_start = self.bottom + 1 - count;
            for y in blank_start..=self.bottom {
                self.rows[y].clear(self.erase_style);
            }
            if blank_start > row {
                self.break_wrap(blank_start - 1);
            }
        }
        self.cursor.wrap_pending = false;
        true
    }

    pub fn erase_line(&mut self, row: usize, mode: u16) {
        let columns = self.rows[row].cells.len();
        let start = if mode == 0 { self.cursor.column } else { 0 };
        let end = if mode == 1 {
            self.cursor.column + 1
        } else {
            columns
        };
        let (start, end) = self.erase(row, start, end);
        // Erased physical boundaries must not join surviving logical lines.
        if start == 0 {
            self.detach_before(row);
        }
        if end == columns {
            self.break_wrap(row);
        }
        self.cursor.wrap_pending = false;
    }

    pub fn erase_display(&mut self, mode: u16) {
        let row = self.cursor.row;
        match mode {
            0 => {
                self.erase_line(row, 0);
                for y in row + 1..self.rows.len() {
                    self.erase_line(y, 2);
                }
            }
            1 => {
                for y in 0..row {
                    self.erase_line(y, 2);
                }
                self.erase_line(row, 1);
            }
            2 => {
                for y in 0..self.rows.len() {
                    self.erase_line(y, 2);
                }
            }
            // The alternate screen has no history. Hidden primary is untouched.
            3 => self.history.clear(),
            _ => unreachable!("validated ED mode"),
        }
    }

    fn break_wrap(&mut self, row: usize) {
        self.rows[row].soft_wrapped = false;
        if self.rows[row]
            .cells
            .last()
            .is_some_and(|c| matches!(c.view(), CellView::WrapPadding))
        {
            *self.rows[row].cells.last_mut().unwrap() = Cell::EMPTY;
        }
    }

    fn detach_before(&mut self, row: usize) {
        if row > 0 {
            self.break_wrap(row - 1);
        } else if let Some(row) = self.history.back_mut() {
            row.soft_wrapped = false;
            if row
                .cells
                .last()
                .is_some_and(|c| matches!(c.view(), CellView::WrapPadding))
            {
                *row.cells.last_mut().unwrap() = Cell::EMPTY;
            }
        }
    }

    pub fn down(&mut self, history_capacity: usize, out: &mut FeedOutcome) {
        if self.cursor.row == self.bottom {
            if self.top != 0 || self.bottom + 1 != self.rows.len() {
                self.detach_before(self.top);
            }
            self.rows[self.top..=self.bottom].rotate_left(1);
            if self.top == 0 && self.bottom + 1 == self.rows.len() && history_capacity > 0 {
                let blank = if self.history.len() == history_capacity {
                    let mut row = self.history.pop_front().unwrap();
                    row.clear(self.erase_style);
                    out.history_evicted = true;
                    row
                } else {
                    let mut row = Row::blank(self.rows[0].cells.len());
                    row.clear(self.erase_style);
                    row
                };
                let old = std::mem::replace(&mut self.rows[self.bottom], blank);
                self.history.push_back(old);
            } else {
                self.rows[self.bottom].clear(self.erase_style);
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
            self.detach_before(self.top);
            self.rows[self.top..=self.bottom].rotate_right(1);
            self.rows[self.top].clear(self.erase_style);
            self.break_wrap(self.bottom);
            out.scrolled_without_history = true;
        } else {
            self.cursor.row = self.cursor.row.saturating_sub(1);
        }
        self.cursor.wrap_pending = false;
        out.changed = true;
    }

    pub fn valid(&self, size: Size, limits: Limits) -> bool {
        if (!self.autowrap && self.cursor.wrap_pending)
            || (!self.saved_cursor.autowrap && self.saved_cursor.cursor.wrap_pending)
            || self.rows.len() != size.lines
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
                        if width == 2
                            && !row.cells.get(i + 1).is_some_and(|c| {
                                matches!(c.view(), CellView::Continuation)
                                    && c.style_id() == cell.style_id()
                            })
                        {
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
                    .filter(|c| c.significant() && !matches!(c.view(), CellView::WrapPadding))
                    .count();
                continue;
            }
            for (x, c) in old.cells.iter().enumerate() {
                if x >= size.columns {
                    outcome.cropped_cells +=
                        usize::from(c.significant() && !matches!(c.view(), CellView::WrapPadding));
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
            .filter(|(_, row)| row.soft_wrapped || row.cells.iter().any(Cell::significant))
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
                    .rposition(Cell::significant)
                    .map_or(0, |i| i + 1)
                    .max(if y == cursor_row { cursor_end } else { 0 })
            };
            for c in &row.cells[..extent] {
                match c.view() {
                    CellView::Empty => {
                        logical_width += 1;
                        logical.push(c.clone());
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
                .filter(|c| c.significant() && !matches!(c.view(), CellView::WrapPadding))
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
                    row.cells
                        .push(Cell::WRAP_PADDING.with_style(cell.style_id()));
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
            let style = cell.style_id();
            row.cells.push(cell);
            if width == 2 {
                row.cells.push(Cell::CONTINUATION.with_style(style));
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
            if last
                .cells
                .last()
                .is_some_and(|c| matches!(c.view(), CellView::WrapPadding))
            {
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
                autowrap: true,
                style: 0,
                erase_style: 0,
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
