//! Storage-only experiment. No terminal parser, product API or unsafe allocator hook.
use std::collections::{HashSet, VecDeque};
use std::mem::size_of;
use std::sync::Arc;

const EMPTY: u32 = 0;
const NARROW: u32 = 1;
const WIDE: u32 = 2;
const TAIL: u32 = 3;
const PAD: u32 = 4;
const SIDE_NARROW: u32 = 5;
const SIDE_WIDE: u32 = 6;
const MASK: u32 = (1 << 29) - 1;
const CHUNK_ROWS: usize = 64;

/// Capacity changes requested by explicitly instrumented containers. Not malloc
/// calls, allocator usable sizes, resident memory or system-wide allocation data.
#[derive(Clone, Copy, Default, Debug)]
pub struct Allocations {
    pub growths: usize,
    pub requested_bytes: usize,
}
impl Allocations {
    fn record<T>(&mut self, before: usize, after: usize) {
        if after > before {
            self.growths += 1;
            self.requested_bytes += after * size_of::<T>();
        }
    }
    pub fn since(self, old: Self) -> Self {
        Self {
            growths: self.growths - old.growths,
            requested_bytes: self.requested_bytes - old.requested_bytes,
        }
    }
}
fn grow<T>(v: &mut Vec<T>, stats: &mut Allocations, f: impl FnOnce(&mut Vec<T>)) {
    let before = v.capacity();
    f(v);
    stats.record::<T>(before, v.capacity());
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub text: Vec<char>,
    pub width: usize,
}
#[derive(Clone)]
pub struct Input {
    pub atoms: Vec<Atom>,
    pub soft: bool,
}
/// Fixed deterministic workloads; construction is outside measured operations.
pub fn input(kind: &str, columns: usize) -> Input {
    let mut atoms = Vec::new();
    let mut x = 0;
    while x < columns {
        let (text, width) = match kind {
            "ascii" => ("x".to_owned(), 1),
            "mixed" if x % 16 == 0 => ("e\u{301}".to_owned(), 1),
            "mixed" if x % 16 == 4 && x + 2 <= columns => ("👩‍💻".to_owned(), 2),
            "mixed" => ("a".to_owned(), 1),
            "dense" if x + 2 <= columns => ("👩‍💻".to_owned(), 2),
            "dense" => ("a".to_owned(), 1),
            "long" => (format!("a{}", "\u{301}".repeat(63)), 1),
            _ => panic!("unknown workload"),
        };
        atoms.push(Atom {
            text: text.chars().collect(),
            width,
        });
        x += width;
    }
    Input { atoms, soft: false }
}

#[derive(Default)]
struct Slot {
    text: Vec<char>,
    next_free: Option<u32>,
}
/// No interning: each live cluster has one slot. Released slots and their bounded
/// buffers are recycled. IDs remain internal; snapshots copy their own text.
pub struct Arena {
    slots: Vec<Slot>,
    free: Option<u32>,
    limit: usize,
    live: usize,
}
impl Arena {
    fn new(limit: usize) -> Self {
        Self {
            slots: Vec::new(),
            free: None,
            limit,
            live: 0,
        }
    }
    fn insert(&mut self, chars: &[char], stats: &mut Allocations) -> u32 {
        assert!((2..=64).contains(&chars.len()));
        let id = if let Some(id) = self.free {
            self.free = self.slots[id as usize].next_free.take();
            id
        } else {
            assert!(self.slots.len() < self.limit && self.slots.len() <= MASK as usize);
            let id = self.slots.len() as u32;
            grow(&mut self.slots, stats, |v| v.push(Slot::default()));
            id
        };
        let slot = &mut self.slots[id as usize];
        assert!(slot.text.is_empty());
        grow(&mut slot.text, stats, |v| v.extend_from_slice(chars));
        self.live += 1;
        id
    }
    fn release(&mut self, id: u32) {
        let slot = &mut self.slots[id as usize];
        assert!(!slot.text.is_empty());
        slot.text.clear();
        slot.next_free = self.free;
        self.free = Some(id);
        self.live -= 1;
    }
    fn heap_bytes(&self) -> usize {
        self.slots.capacity() * size_of::<Slot>()
            + self
                .slots
                .iter()
                .map(|s| s.text.capacity() * size_of::<char>())
                .sum::<usize>()
    }
    fn trim_free_text(&mut self) {
        for slot in &mut self.slots {
            if slot.text.is_empty() {
                slot.text = Vec::new();
            }
        }
    }
}

pub struct Text<'a> {
    pub first: char,
    pub extra: &'a [char],
}
pub trait StorageCell: Clone + Default {
    fn encode(atom: &Atom, arena: &mut Arena, stats: &mut Allocations) -> Self;
    fn role(&self) -> u32;
    fn text<'a>(&'a self, arena: &'a Arena) -> Option<Text<'a>>;
    fn tail() -> Self;
    fn padding() -> Self;
    fn release(&mut self, arena: &mut Arena);
    fn heap_bytes(&self) -> usize {
        0
    }
    fn copied(&self, stats: &mut Allocations) -> Self {
        let _ = stats;
        self.clone()
    }
}

/// Mirrors current engine layout and per-cluster ownership. Synthetic storage
/// workload omits parsing; equality of layouts is checked against the real types.
#[derive(Clone)]
pub struct DeepCluster {
    first: char,
    extra: Vec<char>,
}
#[derive(Clone, Default)]
pub enum DeepCell {
    #[default]
    Empty,
    Lead {
        cluster: DeepCluster,
        width: u8,
    },
    Tail,
    Pad,
}
impl StorageCell for DeepCell {
    fn encode(atom: &Atom, _: &mut Arena, stats: &mut Allocations) -> Self {
        let mut extra = Vec::new();
        // Match incremental core extension rather than preallocating exact length.
        for c in &atom.text[1..] {
            grow(&mut extra, stats, |v| v.push(*c));
        }
        Self::Lead {
            cluster: DeepCluster {
                first: atom.text[0],
                extra,
            },
            width: atom.width as u8,
        }
    }
    fn role(&self) -> u32 {
        match self {
            Self::Empty => EMPTY,
            Self::Lead { width: 1, .. } => NARROW,
            Self::Lead { .. } => WIDE,
            Self::Tail => TAIL,
            Self::Pad => PAD,
        }
    }
    fn text<'a>(&'a self, _: &'a Arena) -> Option<Text<'a>> {
        match self {
            Self::Lead { cluster, .. } => Some(Text {
                first: cluster.first,
                extra: &cluster.extra,
            }),
            _ => None,
        }
    }
    fn tail() -> Self {
        Self::Tail
    }
    fn padding() -> Self {
        Self::Pad
    }
    fn release(&mut self, _: &mut Arena) {
        *self = Self::Empty;
    }
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Lead { cluster, .. } => cluster.extra.capacity() * size_of::<char>(),
            _ => 0,
        }
    }
    fn copied(&self, stats: &mut Allocations) -> Self {
        let result = self.clone();
        if let Self::Lead { cluster, .. } = &result {
            stats.record::<char>(0, cluster.extra.capacity());
        }
        result
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Cell8 {
    word: u32,
    style_id: u32,
}
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Cell16 {
    word: u32,
    style_id: u32,
    hyperlink_id: u32,
    attributes: u32,
}
trait Packed: Clone + Default {
    fn word(&self) -> u32;
    fn from_word(word: u32) -> Self;
}
impl Packed for Cell8 {
    fn word(&self) -> u32 {
        self.word
    }
    fn from_word(word: u32) -> Self {
        Self {
            word,
            ..Self::default()
        }
    }
}
impl Packed for Cell16 {
    fn word(&self) -> u32 {
        self.word
    }
    fn from_word(word: u32) -> Self {
        Self {
            word,
            ..Self::default()
        }
    }
}
impl<C: Packed> StorageCell for C {
    fn encode(atom: &Atom, arena: &mut Arena, stats: &mut Allocations) -> Self {
        let (role, payload) = if atom.text.len() == 1 {
            (atom.width as u32, atom.text[0] as u32)
        } else {
            (
                if atom.width == 1 {
                    SIDE_NARROW
                } else {
                    SIDE_WIDE
                },
                arena.insert(&atom.text, stats),
            )
        };
        Self::from_word((role << 29) | payload)
    }
    fn role(&self) -> u32 {
        match self.word() >> 29 {
            SIDE_NARROW => NARROW,
            SIDE_WIDE => WIDE,
            role => role,
        }
    }
    fn text<'a>(&'a self, arena: &'a Arena) -> Option<Text<'a>> {
        let payload = self.word() & MASK;
        match self.word() >> 29 {
            NARROW | WIDE => Some(Text {
                first: char::from_u32(payload).unwrap(),
                extra: &[],
            }),
            SIDE_NARROW | SIDE_WIDE => {
                let text = &arena.slots[payload as usize].text;
                Some(Text {
                    first: text[0],
                    extra: &text[1..],
                })
            }
            _ => None,
        }
    }
    fn tail() -> Self {
        Self::from_word(TAIL << 29)
    }
    fn padding() -> Self {
        Self::from_word(PAD << 29)
    }
    fn release(&mut self, arena: &mut Arena) {
        if matches!(self.word() >> 29, SIDE_NARROW | SIDE_WIDE) {
            arena.release(self.word() & MASK);
        }
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Default)]
struct Meta {
    id: u64,
    generation: u64,
    soft: bool,
}
struct Row<C> {
    cells: Vec<C>,
    meta: Meta,
}
struct Chunk<C> {
    cells: Vec<C>,
    meta: Vec<Meta>,
}
enum Rows<C> {
    Deep(VecDeque<Row<C>>),
    Chunked {
        chunks: Vec<Chunk<C>>,
        head: usize,
        len: usize,
    },
}
impl<C: StorageCell> Rows<C> {
    fn new(chunked: bool) -> Self {
        if chunked {
            Self::Chunked {
                chunks: Vec::new(),
                head: 0,
                len: 0,
            }
        } else {
            Self::Deep(VecDeque::new())
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Deep(v) => v.len(),
            Self::Chunked { len, .. } => *len,
        }
    }
    fn row(&self, row: usize, columns: usize, capacity: usize) -> (&[C], Meta) {
        match self {
            Self::Deep(v) => (&v[row].cells, v[row].meta),
            Self::Chunked { chunks, head, .. } => {
                let slot = (*head + row) % capacity;
                let chunk = &chunks[slot / CHUNK_ROWS];
                let local = slot % CHUNK_ROWS;
                (
                    &chunk.cells[local * columns..(local + 1) * columns],
                    chunk.meta[local],
                )
            }
        }
    }
    fn row_mut(&mut self, row: usize, columns: usize, capacity: usize) -> (&mut [C], &mut Meta) {
        match self {
            Self::Deep(v) => {
                let r = &mut v[row];
                (&mut r.cells, &mut r.meta)
            }
            Self::Chunked { chunks, head, .. } => {
                let slot = (*head + row) % capacity;
                let chunk = &mut chunks[slot / CHUNK_ROWS];
                let local = slot % CHUNK_ROWS;
                (
                    &mut chunk.cells[local * columns..(local + 1) * columns],
                    &mut chunk.meta[local],
                )
            }
        }
    }
    fn append(&mut self, columns: usize, capacity: usize, stats: &mut Allocations) {
        match self {
            Self::Deep(v) => {
                assert!(v.len() < capacity);
                let mut cells = Vec::new();
                grow(&mut cells, stats, |v| v.resize(columns, C::default()));
                let before = v.capacity();
                v.push_back(Row {
                    cells,
                    meta: Meta::default(),
                });
                stats.record::<Row<C>>(before, v.capacity());
            }
            Self::Chunked { chunks, head, len } => {
                assert!(*len < capacity);
                let slot = (*head + *len) % capacity;
                if slot / CHUNK_ROWS == chunks.len() {
                    let count = CHUNK_ROWS.min(capacity - chunks.len() * CHUNK_ROWS);
                    let mut cells = Vec::new();
                    let mut meta = Vec::new();
                    grow(&mut cells, stats, |v| {
                        v.resize(count * columns, C::default())
                    });
                    grow(&mut meta, stats, |v| v.resize(count, Meta::default()));
                    grow(chunks, stats, |v| v.push(Chunk { cells, meta }));
                }
                *len += 1;
            }
        }
    }
    fn pop_front(&mut self, capacity: usize) {
        match self {
            Self::Deep(v) => {
                v.pop_front().unwrap();
            }
            Self::Chunked { head, len, .. } => {
                *head = (*head + 1) % capacity;
                *len -= 1;
            }
        }
    }
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Deep(v) => {
                v.capacity() * size_of::<Row<C>>()
                    + v.iter()
                        .map(|r| {
                            r.cells.capacity() * size_of::<C>()
                                + r.cells.iter().map(StorageCell::heap_bytes).sum::<usize>()
                        })
                        .sum::<usize>()
            }
            Self::Chunked { chunks, .. } => {
                chunks.capacity() * size_of::<Chunk<C>>()
                    + chunks
                        .iter()
                        .map(|c| {
                            c.cells.capacity() * size_of::<C>()
                                + c.meta.capacity() * size_of::<Meta>()
                        })
                        .sum::<usize>()
            }
        }
    }
}

pub struct Grid<C> {
    rows: Rows<C>,
    arena: Arena,
    pub stats: Allocations,
    columns: usize,
    capacity: usize,
    chunked: bool,
    generation: u64,
}
impl<C: StorageCell> Grid<C> {
    pub fn new(columns: usize, capacity: usize, chunked: bool) -> Self {
        assert!(columns >= 2 && capacity > 0 && columns * capacity <= 4_000_000);
        Self {
            rows: Rows::new(chunked),
            arena: Arena::new(columns * capacity),
            stats: Allocations::default(),
            columns,
            capacity,
            chunked,
            generation: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn heap_bytes(&self) -> usize {
        self.rows.heap_bytes() + self.arena.heap_bytes()
    }
    pub fn live_clusters(&self) -> usize {
        self.arena.live
    }
    pub fn arena_slots(&self) -> usize {
        self.arena.slots.len()
    }
    pub fn trim_free_text(&mut self) {
        self.arena.trim_free_text();
    }
    fn release_row(&mut self, row: usize) {
        let (cells, _) = self.rows.row_mut(row, self.columns, self.capacity);
        for cell in cells {
            cell.release(&mut self.arena);
        }
    }
    pub fn push(&mut self, input: &Input) {
        if self.len() == self.capacity {
            self.release_row(0);
            self.rows.pop_front(self.capacity);
        }
        self.rows
            .append(self.columns, self.capacity, &mut self.stats);
        self.replace(self.len() - 1, input, true);
    }
    pub fn replace(&mut self, row: usize, input: &Input, new_id: bool) {
        assert_eq!(
            input.atoms.iter().map(|a| a.width).sum::<usize>(),
            self.columns
        );
        assert!(
            input
                .atoms
                .iter()
                .all(|a| (1..=2).contains(&a.width) && (1..=64).contains(&a.text.len()))
        );
        self.release_row(row);
        self.generation += 1;
        let (cells, meta) = self.rows.row_mut(row, self.columns, self.capacity);
        let mut x = 0;
        for atom in &input.atoms {
            cells[x] = C::encode(atom, &mut self.arena, &mut self.stats);
            if atom.width == 2 {
                cells[x + 1] = C::tail();
            }
            x += atom.width;
        }
        *meta = Meta {
            id: if new_id { self.generation } else { meta.id },
            generation: self.generation,
            soft: input.soft,
        };
    }
    /// Storage repacking only: no terminal cursor/history viewport policy. Keeps
    /// every retained atom and hard break. New rows coexist with old rows; packed
    /// side IDs transfer ownership without cloning or releasing their arena text.
    pub fn repack(&mut self, columns: usize) -> usize {
        assert!(columns >= 2);
        let before = self.heap_bytes();
        let capacity = self.len() * (1 + self.columns / (columns - 1));
        let mut out = Rows::<C>::new(self.chunked);
        let mut x = 0;
        let mut open = false;
        for y in 0..self.len() {
            let (old, meta) = self.rows.row(y, self.columns, self.capacity);
            if !open {
                out.append(columns, capacity, &mut self.stats);
                open = true;
            }
            let extent = if meta.soft {
                old.len()
            } else {
                old.iter()
                    .rposition(|c| !matches!(c.role(), EMPTY | PAD))
                    .map_or(0, |i| i + 1)
            };
            for cell in &old[..extent] {
                let role = cell.role();
                if matches!(role, TAIL | PAD) {
                    continue;
                }
                let width = if role == WIDE { 2 } else { 1 };
                if x + width > columns {
                    let n = out.len() - 1;
                    let (cells, meta) = out.row_mut(n, columns, capacity);
                    if x < columns {
                        cells[x] = C::padding();
                    }
                    meta.soft = true;
                    out.append(columns, capacity, &mut self.stats);
                    x = 0;
                }
                let n = out.len() - 1;
                let (cells, _) = out.row_mut(n, columns, capacity);
                cells[x] = cell.copied(&mut self.stats);
                if width == 2 {
                    cells[x + 1] = C::tail();
                }
                x += width;
            }
            if !meta.soft {
                open = false;
                x = 0;
            }
        }
        for y in 0..out.len() {
            self.generation += 1;
            let (_, meta) = out.row_mut(y, columns, capacity);
            meta.id = self.generation;
            meta.generation = self.generation;
        }
        // Both row sets are live, while the packed arena is shared once.
        let peak = before + out.heap_bytes();
        self.rows = out;
        self.columns = columns;
        self.capacity = capacity;
        peak
    }
    pub fn snapshot(&self, visible: usize, previous: Option<&Snapshot>) -> (Snapshot, Allocations) {
        let mut stats = Allocations::default();
        let mut rows = Vec::new();
        let start = self.len().saturating_sub(visible);
        grow(&mut rows, &mut stats, |v| {
            v.reserve_exact(self.len() - start)
        });
        for y in start..self.len() {
            let (cells, meta) = self.rows.row(y, self.columns, self.capacity);
            let reused = previous.and_then(|s| {
                s.rows
                    .iter()
                    .find(|r| r.meta.id == meta.id && r.meta.generation == meta.generation)
            });
            let row = if let Some(row) = reused {
                Arc::clone(row)
            } else {
                let mut output = Vec::new();
                grow(&mut output, &mut stats, |v| v.reserve_exact(cells.len()));
                let mut text = Vec::new();
                for cell in cells {
                    let role = cell.role();
                    let (payload, length) = if let Some(chars) = cell.text(&self.arena) {
                        if chars.extra.is_empty() {
                            (chars.first as u32, 0)
                        } else {
                            let offset = text.len() as u32;
                            grow(&mut text, &mut stats, |v| v.push(chars.first));
                            grow(&mut text, &mut stats, |v| v.extend_from_slice(chars.extra));
                            (offset, (chars.extra.len() + 1) as u32)
                        }
                    } else {
                        (0, 0)
                    };
                    grow(&mut output, &mut stats, |v| {
                        v.push(SnapshotCell {
                            payload,
                            length,
                            role,
                            style_id: 0,
                        })
                    });
                }
                stats.record::<SnapshotRow>(0, 1); // Arc payload only; header explicitly excluded.
                Arc::new(SnapshotRow {
                    meta,
                    cells: output,
                    text,
                })
            };
            grow(&mut rows, &mut stats, |v| v.push(row));
        }
        (Snapshot { rows }, stats)
    }
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut add = |value: u64| {
            hash = (hash ^ value).wrapping_mul(0x100000001b3);
        };
        for y in 0..self.len() {
            let (cells, meta) = self.rows.row(y, self.columns, self.capacity);
            add(u64::from(meta.soft));
            for cell in cells {
                add(cell.role() as u64);
                add(0); // Reserved style IDs are all zero in this experiment.
                if let Some(text) = cell.text(&self.arena) {
                    add(text.first as u64);
                    for c in text.extra {
                        add(*c as u64);
                    }
                } else {
                    add(0);
                }
                add(u64::MAX);
            }
        }
        hash
    }
    /// For the deep baseline this reproduces visible Row/Cell cloning. Packed
    /// IDs are not independently owned, so this measurement is baseline-only.
    pub fn baseline_clone_drop(&self, visible: usize) -> (usize, Allocations) {
        assert_eq!(size_of::<C>(), size_of::<DeepCell>());
        let mut stats = Allocations::default();
        let mut rows = Vec::new();
        for y in self.len().saturating_sub(visible)..self.len() {
            let (source, meta) = self.rows.row(y, self.columns, self.capacity);
            let mut cells = Vec::new();
            grow(&mut cells, &mut stats, |v| v.reserve_exact(source.len()));
            for c in source {
                cells.push(c.copied(&mut stats));
            }
            grow(&mut rows, &mut stats, |v| v.push(Row { cells, meta }));
        }
        let bytes = rows.capacity() * size_of::<Row<C>>()
            + rows
                .iter()
                .map(|r| {
                    r.cells.capacity() * size_of::<C>()
                        + r.cells.iter().map(StorageCell::heap_bytes).sum::<usize>()
                })
                .sum::<usize>();
        std::hint::black_box(&rows);
        (bytes, stats)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
struct SnapshotCell {
    payload: u32,
    length: u32,
    role: u32,
    style_id: u32,
}
struct SnapshotRow {
    meta: Meta,
    cells: Vec<SnapshotCell>,
    text: Vec<char>,
}
pub struct Snapshot {
    rows: Vec<Arc<SnapshotRow>>,
}
impl Snapshot {
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut add = |value: u64| {
            hash = (hash ^ value).wrapping_mul(0x100000001b3);
        };
        for row in &self.rows {
            add(u64::from(row.meta.soft));
            for cell in &row.cells {
                add(cell.role as u64);
                add(cell.style_id as u64);
                if cell.length == 0 {
                    add(cell.payload as u64);
                } else {
                    for c in &row.text[cell.payload as usize..(cell.payload + cell.length) as usize]
                    {
                        add(*c as u64);
                    }
                }
                add(u64::MAX);
            }
        }
        hash
    }
    /// Count shared row payloads once across bounded in-flight frames. Arc's
    /// private refcount header and allocator metadata are not included.
    pub fn retained_bytes(frames: &[&Self]) -> usize {
        let mut seen = HashSet::new();
        let mut bytes = 0;
        for frame in frames {
            bytes += frame.rows.capacity() * size_of::<Arc<SnapshotRow>>();
            for row in &frame.rows {
                if seen.insert(Arc::as_ptr(row)) {
                    bytes += size_of::<SnapshotRow>()
                        + row.cells.capacity() * size_of::<SnapshotCell>()
                        + row.text.capacity() * size_of::<char>();
                }
            }
        }
        bytes
    }
}

pub fn layout() -> serde_json::Value {
    serde_json::json!({ "current_engine_cell": size_of::<nebulax_terminal::Cell>(), "current_engine_row": size_of::<nebulax_terminal::Row>(), "baseline_cell": size_of::<DeepCell>(), "cell8": size_of::<Cell8>(), "cell16": size_of::<Cell16>(), "row_metadata": size_of::<Meta>(), "deep_row": size_of::<Row<DeepCell>>(), "snapshot_cell": size_of::<SnapshotCell>() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nebulax_terminal::{Cell, Limits, Size, Terminal, WidthPolicy};

    fn filled<C: StorageCell>(chunked: bool, kind: &str, columns: usize, rows: usize) -> Grid<C> {
        let mut grid = Grid::new(columns, rows, chunked);
        for _ in 0..rows {
            grid.push(&input(kind, columns));
        }
        grid
    }
    fn core_checksum(t: &Terminal) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut add = |v: u64| {
            hash = (hash ^ v).wrapping_mul(0x100000001b3);
        };
        for row in t.screen() {
            add(u64::from(row.soft_wrapped()));
            for cell in row.cells() {
                add(match cell {
                    Cell::Empty => EMPTY,
                    Cell::Lead { width, .. } => u32::from(*width),
                    Cell::Continuation => TAIL,
                    Cell::WrapPadding => PAD,
                } as u64);
                add(0);
                if let Cell::Lead { cluster, .. } = cell {
                    for c in cluster.chars() {
                        add(c as u64);
                    }
                } else {
                    add(0);
                }
                add(u64::MAX);
            }
        }
        hash
    }
    #[test]
    fn layouts_and_synthetic_content_match_current_engine() {
        assert_eq!(size_of::<DeepCell>(), size_of::<Cell>());
        assert_eq!(size_of::<Cell8>(), 8);
        assert_eq!(size_of::<Cell16>(), 16);
        for kind in ["ascii", "mixed", "dense", "long"] {
            let source = input(kind, 20);
            let text: String = source.atoms.iter().flat_map(|a| a.text.iter()).collect();
            let mut t = Terminal::new(
                Size {
                    columns: 20,
                    lines: 3,
                },
                Limits::default(),
                WidthPolicy::default(),
            )
            .unwrap();
            for i in 0..3 {
                if i > 0 {
                    t.feed(b"\r\n");
                }
                t.feed(text.as_bytes());
            }
            let expected = core_checksum(&t);
            assert_eq!(filled::<DeepCell>(false, kind, 20, 3).checksum(), expected);
            assert_eq!(filled::<Cell8>(true, kind, 20, 3).checksum(), expected);
            assert_eq!(filled::<Cell16>(true, kind, 20, 3).checksum(), expected);
        }
    }
    #[test]
    fn lazy_chunks_and_steady_scroll_reuse_do_not_grow() {
        let mut grid = Grid::<Cell16>::new(80, 130, true);
        assert_eq!(grid.heap_bytes(), 0);
        let source = input("mixed", 80);
        for _ in 0..130 {
            grid.push(&source);
        }
        let bytes = grid.heap_bytes();
        let slots = grid.arena_slots();
        let stats = grid.stats;
        let checksum = grid.checksum();
        for _ in 0..1000 {
            grid.push(&source);
        }
        assert_eq!(grid.heap_bytes(), bytes);
        assert_eq!(grid.arena_slots(), slots);
        assert_eq!(grid.stats.since(stats).growths, 0);
        assert_eq!(grid.checksum(), checksum);
    }
    #[test]
    fn snapshots_survive_id_reuse_repack_trim_and_thread_transfer() {
        let mut grid = filled::<Cell8>(true, "dense", 20, 65);
        let (old, _) = grid.snapshot(40, None);
        let checksum = old.checksum();
        for _ in 0..130 {
            grid.push(&input("long", 20));
        }
        grid.repack(7);
        grid.repack(20);
        for row in 0..grid.len() {
            grid.replace(row, &input("ascii", 20), false);
        }
        grid.trim_free_text();
        assert_eq!(grid.live_clusters(), 0);
        drop(grid);
        assert_eq!(
            std::thread::spawn(move || old.checksum()).join().unwrap(),
            checksum
        );
    }
    #[test]
    fn damage_reuses_only_unchanged_rows_and_survives_row_moves() {
        let mut grid = filled::<Cell16>(true, "mixed", 20, 65);
        let (old, _) = grid.snapshot(40, None);
        let (same, stats) = grid.snapshot(40, Some(&old));
        assert!(
            old.rows
                .iter()
                .zip(&same.rows)
                .all(|(a, b)| Arc::ptr_eq(a, b))
        );
        assert!(stats.growths < 10); // Only the frame's row-pointer vector grows.
        grid.replace(64, &input("ascii", 20), false);
        let (edited, _) = grid.snapshot(40, Some(&old));
        assert_eq!(
            old.rows
                .iter()
                .zip(&edited.rows)
                .filter(|(a, b)| !Arc::ptr_eq(a, b))
                .count(),
            1
        );
        grid.push(&input("ascii", 20));
        let (scrolled, _) = grid.snapshot(40, Some(&edited));
        assert!(
            edited.rows[1..]
                .iter()
                .zip(&scrolled.rows)
                .all(|(a, b)| Arc::ptr_eq(a, b))
        );
        assert!(!Arc::ptr_eq(&edited.rows[39], &scrolled.rows[39]));
    }
    #[test]
    fn repacking_preserves_hard_lines_and_matches_all_layouts() {
        for kind in ["ascii", "mixed", "dense", "long"] {
            let mut a = filled::<DeepCell>(false, kind, 20, 5);
            let mut b = filled::<Cell8>(true, kind, 20, 5);
            let mut c = filled::<Cell16>(true, kind, 20, 5);
            let original = a.checksum();
            for width in [3, 20, 7, 20, 2, 20] {
                a.repack(width);
                b.repack(width);
                c.repack(width);
                assert_eq!(a.checksum(), b.checksum());
                assert_eq!(a.checksum(), c.checksum());
                if width == 20 {
                    assert_eq!(a.checksum(), original);
                }
            }
        }
    }
    #[test]
    fn arena_exhaustion_is_explicit_and_released_ids_are_reusable() {
        let mut arena = Arena::new(1);
        let mut stats = Allocations::default();
        let id = arena.insert(&['a', '\u{301}'], &mut stats);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || arena.insert(&['b', '\u{301}'], &mut stats)
            ))
            .is_err()
        );
        arena.release(id);
        assert_eq!(arena.insert(&['c', '\u{301}'], &mut stats), id);
        assert_eq!(arena.live, 1);
        arena.release(id);
        let before = arena.heap_bytes();
        arena.trim_free_text();
        assert!(arena.heap_bytes() < before);
    }
}
