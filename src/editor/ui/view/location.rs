#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub grapheme_idx: usize,
    pub line_idx: usize,
}
