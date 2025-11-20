#[derive(Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            row: self.row.saturating_sub(other.row),
            col: self.col.saturating_sub(other.col),
        }
    }
}
