use super::ByteIdx;
use super::GraphemeWidth;

#[derive(Debug)]
pub struct TextFragment {
    pub start_byte_idx: ByteIdx,
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    pub replacement: Option<char>,
}
