//! Owned visible frames. No engine borrows, cluster handles, history or I/O.
use crate::{CellView, Cursor, Size, Terminal, style::Style};

pub const MAX_SNAPSHOT_BYTES: usize = 2 * 1024 * 1024;

/// Private C bridge wire cell. UTF-8 offsets reference this frame's text only.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnapshotCell {
    pub text_offset: u32,
    pub text_len: u16,
    pub width: u8,
    /// 0 empty, 1 lead, 2 continuation, 3 wrap padding.
    pub kind: u8,
    pub style_id: u16,
    pub reserved: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    ByteLimit,
    GenerationExhausted,
}
#[derive(Debug)]
pub struct Snapshot {
    size: Size,
    cursor: Cursor,
    cursor_visible: bool,
    alternate: bool,
    generation: u64,
    cells: Box<[SnapshotCell]>,
    text: Box<[u8]>,
    row_versions: Box<[u64]>,
    row_wraps: Box<[u8]>,
    styles: Box<[Style]>,
}
impl Snapshot {
    /// Previous must be the immediately preceding published frame for this
    /// session. Renderers compare row versions to their OWN last displayed frame,
    /// since the latest-frame mailbox may skip intermediate generations.
    pub fn capture(terminal: &Terminal, previous: Option<&Self>) -> Result<Self, SnapshotError> {
        let generation = previous
            .map_or(Some(1), |p| p.generation.checked_add(1))
            .ok_or(SnapshotError::GenerationExhausted)?;
        let size = terminal.size();
        let count = size.columns * size.lines; // Validated terminal geometry.
        let metadata = count * std::mem::size_of::<SnapshotCell>()
            + size.lines * 9
            + terminal.styles.len() * std::mem::size_of::<Style>();
        let mut text_bytes = 0usize;
        for row in terminal.screen() {
            for cell in row.cells() {
                if let CellView::Lead { cluster, .. } = cell.view() {
                    text_bytes += cluster.chars().map(char::len_utf8).sum::<usize>();
                    if metadata + text_bytes > MAX_SNAPSHOT_BYTES {
                        return Err(SnapshotError::ByteLimit);
                    }
                }
            }
        }
        if metadata + text_bytes > MAX_SNAPSHOT_BYTES {
            return Err(SnapshotError::ByteLimit);
        }
        // Exact sizing avoids geometric text-capacity growth and per-cluster
        // allocations. These are owned payload bounds, not allocator/RSS claims.
        let mut cells = Vec::with_capacity(count);
        let mut text = Vec::with_capacity(text_bytes);
        let mut row_wraps = Vec::with_capacity(size.lines);
        for row in terminal.screen() {
            row_wraps.push(u8::from(row.soft_wrapped()));
            for cell in row.cells() {
                let mut view = SnapshotCell {
                    style_id: cell.style_id(),
                    ..SnapshotCell::default()
                };
                match cell.view() {
                    CellView::Empty => {}
                    CellView::Continuation => view.kind = 2,
                    CellView::WrapPadding => view.kind = 3,
                    CellView::Lead { cluster, width } => {
                        view.kind = 1;
                        view.width = width;
                        view.text_offset = text.len() as u32;
                        for c in cluster.chars() {
                            text.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                        }
                        view.text_len = (text.len() - view.text_offset as usize) as u16;
                    }
                }
                cells.push(view);
            }
        }
        let mut result = Self {
            size,
            cursor: terminal.cursor(),
            cursor_visible: terminal.cursor_visible(),
            alternate: terminal.is_alternate(),
            generation,
            cells: cells.into_boxed_slice(),
            text: text.into_boxed_slice(),
            row_versions: vec![generation; size.lines].into_boxed_slice(),
            row_wraps: row_wraps.into_boxed_slice(),
            styles: terminal.styles.palette().collect(),
        };
        if let Some(old) = previous.filter(|p| p.size == size && p.alternate == result.alternate) {
            for row in 0..size.lines {
                if result.row_equal(old, row) {
                    result.row_versions[row] = old.row_versions[row];
                }
            }
        }
        Ok(result)
    }
    fn row_equal(&self, old: &Self, row: usize) -> bool {
        let range = row * self.size.columns..(row + 1) * self.size.columns;
        self.row_wraps[row] == old.row_wraps[row]
            && self.cells[range.clone()]
                .iter()
                .zip(&old.cells[range])
                .all(|(a, b)| {
                    a.kind == b.kind
                        && a.width == b.width
                        && self.cell_text(a) == old.cell_text(b)
                        && self.cell_style(a) == old.cell_style(b)
                })
    }
    pub fn styles(&self) -> &[Style] {
        &self.styles
    }
    pub fn cell_style(&self, cell: &SnapshotCell) -> Option<&Style> {
        self.styles.get(usize::from(cell.style_id))
    }
    pub fn cell_text(&self, cell: &SnapshotCell) -> Option<&str> {
        let start = cell.text_offset as usize;
        let end = start.checked_add(cell.text_len as usize)?;
        std::str::from_utf8(self.text.get(start..end)?).ok()
    }
    pub fn size(&self) -> Size {
        self.size
    }
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }
    pub fn is_alternate(&self) -> bool {
        self.alternate
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn cells(&self) -> &[SnapshotCell] {
        &self.cells
    }
    pub fn text(&self) -> &[u8] {
        &self.text
    }
    pub fn row_versions(&self) -> &[u64] {
        &self.row_versions
    }
    pub fn row_wraps(&self) -> &[u8] {
        &self.row_wraps
    }
    pub fn payload_bytes(&self) -> usize {
        std::mem::size_of_val(&*self.cells)
            + self.text.len()
            + self.row_versions.len() * 8
            + self.row_wraps.len()
            + std::mem::size_of_val(&*self.styles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, WidthPolicy};
    #[test]
    fn generation_never_wraps_into_a_stale_row_version() {
        let t = Terminal::new(
            Size {
                columns: 2,
                lines: 1,
            },
            Limits::default(),
            WidthPolicy::default(),
        )
        .unwrap();
        let mut frame = Snapshot::capture(&t, None).unwrap();
        frame.generation = u64::MAX;
        assert_eq!(
            Snapshot::capture(&t, Some(&frame)).unwrap_err(),
            SnapshotError::GenerationExhausted
        );
    }
}
