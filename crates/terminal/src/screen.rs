//! Owned screen storage and bounded resize. Original implementation.
use crate::{Cell, Cursor, Limits, ResizeOutcome, Row, Size};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Screen {
    pub rows: Vec<Row>,
    pub history: VecDeque<Row>,
    pub cursor: Cursor,
}

impl Screen {
    pub fn new(size: Size) -> Self {
        Self {
            rows: (0..size.lines).map(|_| Row::blank(size.columns)).collect(),
            history: VecDeque::new(),
            cursor: Cursor::default(),
        }
    }

    pub fn valid(&self, size: Size, limits: Limits) -> bool {
        if self.rows.len() != size.lines
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
                match cell {
                    Cell::Lead { cluster, width } => {
                        if !matches!(width, 1 | 2)
                            || cluster.scalar_count() > limits.cluster_scalars
                        {
                            return false;
                        }
                        if *width == 2 && row.cells.get(i + 1) != Some(&Cell::Continuation) {
                            return false;
                        }
                    }
                    Cell::Continuation => {
                        if i == 0 || !matches!(row.cells[i - 1], Cell::Lead { width: 2, .. }) {
                            return false;
                        }
                    }
                    Cell::WrapPadding => {
                        if i + 1 != size.columns {
                            return false;
                        }
                    }
                    Cell::Empty => {}
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
                    .filter(|c| !matches!(c, Cell::Empty | Cell::WrapPadding))
                    .count();
                continue;
            }
            for (x, c) in old.cells.iter().enumerate() {
                if x >= size.columns {
                    outcome.cropped_cells +=
                        usize::from(!matches!(c, Cell::Empty | Cell::WrapPadding));
                    continue;
                }
                result.rows[y].cells[x] = match c {
                    Cell::Lead { width: 2, .. } if x + 1 == size.columns => {
                        outcome.cropped_cells += 1;
                        Cell::Empty
                    }
                    Cell::WrapPadding => Cell::Empty,
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
                        .any(|c| !matches!(c, Cell::Empty | Cell::WrapPadding))
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
                    .filter(|c| !matches!(c, Cell::WrapPadding))
                    .count();
                anchor = Some((logical_width + offset, self.cursor.wrap_pending));
            }
            let extent = if row.soft_wrapped {
                row.cells.len()
            } else {
                row.cells
                    .iter()
                    .rposition(|c| !matches!(c, Cell::Empty | Cell::WrapPadding))
                    .map_or(0, |i| i + 1)
                    .max(if y == cursor_row { cursor_end } else { 0 })
            };
            for c in &row.cells[..extent] {
                match c {
                    Cell::Empty => {
                        logical_width += 1;
                        logical.push(Cell::Empty);
                    }
                    Cell::Lead { width, .. } => {
                        logical_width += usize::from(*width);
                        logical.push(c.clone());
                    }
                    Cell::Continuation | Cell::WrapPadding => {}
                }
            }
            if !row.soft_wrapped || y + 1 == significant {
                sink.line(logical.drain(..), anchor.take());
                logical_width = 0;
            }
        }
        sink.finish()
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
                .filter(|c| !matches!(c, Cell::Empty | Cell::WrapPadding))
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
            let width = match &cell {
                Cell::Lead { width, .. } => usize::from(*width),
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
                    row.cells.push(Cell::WrapPadding);
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
                row.cells.push(Cell::Continuation);
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
            if last.cells.last() == Some(&Cell::WrapPadding) {
                *last.cells.last_mut().unwrap() = Cell::Empty;
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
                row.cells.resize(self.size.columns, Cell::Empty);
                row
            })
            .collect();
        cursor.row -= start;
        let mut rows: Vec<_> = self
            .rows
            .into_iter()
            .take(self.size.lines)
            .map(|mut row| {
                row.cells.resize(self.size.columns, Cell::Empty);
                row
            })
            .collect();
        rows.resize_with(self.size.lines, || Row::blank(self.size.columns));
        (
            Screen {
                rows,
                history,
                cursor,
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
        assert!(t.screen()[0].cells().iter().all(|c| *c == Cell::Empty));
        assert_eq!(t.cursor(), Cursor::default());
        t.resize(Size {
            columns: 8,
            lines: 4,
        })
        .unwrap();
        assert!(matches!(t.screen()[0].cells()[0], Cell::Lead { .. }));
        assert!(!t.screen()[0].soft_wrapped());
        assert!(
            t.screen()[1..]
                .iter()
                .flat_map(Row::cells)
                .all(|c| *c == Cell::Empty)
        );
        assert!(t.invariants_hold());
    }
}
