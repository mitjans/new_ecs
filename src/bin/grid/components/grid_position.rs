#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct GridPosition {
    pub row: usize,
    pub col: usize,
}

impl GridPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn manhattan(&self, other: &GridPosition) -> usize {
        self.row.abs_diff(other.row) + self.col.abs_diff(other.col)
    }
}
