//! Compact directly owned cells. No global arena, handles, interning or unsafe.

/// A borrowed logical view; neither this view nor its text escapes a grid borrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cluster<'a> {
    first: char,
    extra: &'a [char],
}
impl Cluster<'_> {
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        std::iter::once(self.first).chain(self.extra.iter().copied())
    }
    pub fn scalar_count(&self) -> usize {
        1 + self.extra.len()
    }
    pub(crate) fn last(&self) -> char {
        self.extra.last().copied().unwrap_or(self.first)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellView<'a> {
    Empty,
    Lead { cluster: Cluster<'a>, width: u8 },
    Continuation,
    WrapPadding,
}

/// Sixteen bytes on the supported 64-bit target, including a reserved style ID.
/// The first scalar is inline. Only multi-scalar leads allocate a tail. Boxing
/// its Vec deliberately moves the 24-byte header out of every ordinary cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    #[allow(clippy::box_collection)]
    extra: Option<Box<Vec<char>>>,
    first: char,
    style: u16,
    kind: u8,
    width: u8,
}
impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}
impl Cell {
    pub(crate) const EMPTY: Self = Self {
        extra: None,
        first: '\0',
        style: 0,
        kind: 0,
        width: 0,
    };
    pub(crate) const CONTINUATION: Self = Self {
        kind: 2,
        ..Self::EMPTY
    };
    pub(crate) const WRAP_PADDING: Self = Self {
        kind: 3,
        ..Self::EMPTY
    };
    pub(crate) fn lead(first: char, width: u8) -> Self {
        Self {
            first,
            width,
            kind: 1,
            ..Self::EMPTY
        }
    }
    pub fn view(&self) -> CellView<'_> {
        match self.kind {
            0 => CellView::Empty,
            1 => CellView::Lead {
                cluster: Cluster {
                    first: self.first,
                    extra: self.extra.as_deref().map_or(&[], Vec::as_slice),
                },
                width: self.width,
            },
            2 => CellView::Continuation,
            3 => CellView::WrapPadding,
            _ => unreachable!("private cell kind"),
        }
    }
    /// Engine-local style ID; only meaningful with the owning terminal.
    pub fn style_id(&self) -> u16 {
        self.style
    }
    pub(crate) fn with_style(mut self, style: u16) -> Self {
        self.style = style;
        self
    }
    pub(crate) fn significant(&self) -> bool {
        self.style != 0 || !matches!(self.view(), CellView::Empty | CellView::WrapPadding)
    }
    pub(crate) fn push(&mut self, scalar: char) {
        debug_assert_eq!(self.kind, 1);
        let extra = self
            .extra
            .get_or_insert_with(|| Box::new(Vec::with_capacity(4)));
        debug_assert!(extra.len() < 63);
        if extra.len() == extra.capacity() {
            // Cloning may leave a non-power-of-two capacity. Ordinary doubling
            // could retain 80/120 scalars for a tail capped at 63.
            let target = (extra.capacity() * 2).clamp(4, 63);
            extra.reserve_exact(target - extra.len());
        }
        extra.push(scalar);
    }
    pub(crate) fn set_width(&mut self, width: u8) {
        self.width = width;
    }
    pub(crate) fn heap_bytes(&self) -> usize {
        self.extra.as_ref().map_or(0, |v| {
            std::mem::size_of::<Vec<char>>() + v.capacity() * std::mem::size_of::<char>()
        })
    }
    pub(crate) fn allocations(&self) -> usize {
        usize::from(self.extra.is_some()) * 2
    }
    pub(crate) fn valid(&self) -> bool {
        match self.view() {
            CellView::Lead { cluster, width } => {
                matches!(width, 1 | 2)
                    && cluster.scalar_count() <= 64
                    && self
                        .extra
                        .as_ref()
                        .is_none_or(|v| !v.is_empty() && v.capacity() <= 63)
            }
            _ => self.extra.is_none() && self.first == '\0' && self.width == 0,
        }
    }
}
