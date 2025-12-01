#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}
