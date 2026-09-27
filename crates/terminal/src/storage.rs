//! Accounted grid container capacity, excluding allocator overhead and snapshots.
use crate::{Cell, Row, Terminal};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StorageUsage {
    pub cell_bytes: usize,
    pub row_bytes: usize,
    pub cluster_bytes: usize,
    pub cluster_allocations: usize,
    pub style_bytes: usize,
    pub tab_bytes: usize,
}
impl StorageUsage {
    pub fn heap_bytes(self) -> usize {
        self.cell_bytes + self.row_bytes + self.cluster_bytes + self.style_bytes + self.tab_bytes
    }
}
impl Terminal {
    /// Includes active and hidden primary capacities. Excludes parser/output,
    /// transient resize copies, allocator metadata and process RSS.
    pub fn storage_usage(&self) -> StorageUsage {
        let mut result = StorageUsage {
            style_bytes: self.styles.heap_bytes(),
            tab_bytes: self.tabs.heap_bytes(),
            ..StorageUsage::default()
        };
        for screen in std::iter::once(&self.active).chain(self.saved_primary.iter()) {
            result.row_bytes +=
                (screen.rows.capacity() + screen.history.capacity()) * std::mem::size_of::<Row>();
            for row in screen.rows.iter().chain(&screen.history) {
                result.cell_bytes += row.cells.capacity() * std::mem::size_of::<Cell>();
                for cell in &row.cells {
                    result.cluster_bytes += cell.heap_bytes();
                    result.cluster_allocations += cell.allocations();
                }
            }
        }
        result
    }
}
