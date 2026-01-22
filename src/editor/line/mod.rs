mod grapheme_width;
mod text_fragment;

use grapheme_width::GraphemeWidth;
use std::{
    fmt,
    ops::{Deref, Range},
};
use text_fragment::TextFragment;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

type GraphemeIdx = usize;
type ByteIdx = usize;

#[derive(Default)]
pub struct Line {
    string: String,
    fragments: Vec<TextFragment>,
}

impl Line {
    // build a `Line` from a string without \n
    pub fn from(line_str: &str) -> Self {
        debug_assert!(line_str.is_empty() || line_str.lines().count() == 1);
        let source = line_str.to_string();
        let fragments = Self::str_to_fragments(line_str);
        Self {
            string: source,
            fragments,
        }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        let grapheme_to_fragment = |(byte_idx, grapheme): (ByteIdx, &str)| {
            let (replacement, rendered_width) = Self::get_replacement_character(grapheme)
                .map_or_else(
                    || {
                        let unicode_width = grapheme.width();
                        let rendered_width = match unicode_width {
                            0 | 1 => GraphemeWidth::Half,
                            _ => GraphemeWidth::Full,
                        };
                        (None, rendered_width)
                    },
                    |replacement| (Some(replacement), GraphemeWidth::Half),
                );

            TextFragment {
                byte_idx,
                grapheme: grapheme.to_string(),
                rendered_width,
                replacement,
            }
        };

        line_str
            .grapheme_indices(true)
            .map(grapheme_to_fragment)
            .collect()
    }

    fn get_replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();
        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if for_str.chars().all(char::is_control) => Some('▯'),
            _ if width > 0 && for_str.trim().is_empty() => Some('␣'),
            _ if width == 0 => Some('·'),
            _ => None,
        }
    }

    pub fn get_visible_graphemes(&self, range: Range<GraphemeIdx>) -> String {
        let start = range.start;
        let end = range.end;

        let mut result = String::new();
        let mut current_pos = 0;
        for fragment in &self.fragments {
            let fragment_end = usize::from(fragment.rendered_width).saturating_add(current_pos);
            if current_pos >= end {
                break;
            }

            if fragment_end > start {
                // edge case handling
                if fragment_end > end || current_pos < start {
                    result.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }

            current_pos = fragment_end;
        }

        result
    }

    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }

    pub fn width(&self) -> GraphemeIdx {
        self.width_until(self.grapheme_count())
    }

    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> GraphemeIdx {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    // region: edit
    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    // insert a character into the line, or appends it at the end if `at == grapheme_count + 1`
    pub fn insert_char(&mut self, ch: char, at: GraphemeIdx) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.byte_idx, ch);
        } else {
            self.string.push(ch);
        }
        self.rebuild_fragments();
    }

    // delete the character at `at`
    pub fn delete(&mut self, at: GraphemeIdx) {
        debug_assert!(at <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.byte_idx;
            let end = start.saturating_add(fragment.grapheme.len());
            self.string.drain(start..end);
        }
        self.rebuild_fragments();
    }

    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        self.rebuild_fragments();
    }

    pub fn append_char(&mut self, ch: char) {
        self.insert_char(ch, self.grapheme_count());
    }

    pub fn split(&mut self, at: GraphemeIdx) -> Self {
        Self {
            string: self.string.split_off(at),
            fragments: self.fragments.split_off(at),
        }
    }

    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }
    // endregion

    // region: search
    pub fn search_forward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        if self.is_empty() || from_grapheme_idx >= self.grapheme_count() {
            return None;
        }

        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);
        self.string
            .get(start_byte_idx..)
            .and_then(|substr| substr.find(query))
            .map(|byte_idx| {
                self.byte_idx_to_grapheme_idx(byte_idx)
                    .saturating_add(from_grapheme_idx)
            })
    }

    pub fn search_backward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if self.is_empty() || from_grapheme_idx == 0 {
            return None;
        }

        let end_byte_idx = if from_grapheme_idx == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        };

        self.string
            .get(..end_byte_idx)
            .and_then(|substr| substr.match_indices(query).last())
            .map(|(idx, _)| self.byte_idx_to_grapheme_idx(idx))
    }

    // get the grapheme index from byte
    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> GraphemeIdx {
        debug_assert!(byte_idx <= self.string.len());
        self.fragments
            .iter()
            .position(|fragment| fragment.byte_idx >= byte_idx)
            .unwrap_or_else(|| {
                #[cfg(debug_assertions)]
                {
                    panic!("Invalid byte_idx passed to byte_idx_to_grapheme_idx: {byte_idx:?}");
                }
                #[cfg(not(debug_assertions))]
                {
                    0
                }
            })
    }

    // get the start byte from grapheme index
    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        debug_assert!(grapheme_idx <= self.grapheme_count());
        if grapheme_idx == 0 || self.grapheme_count() == 0 {
            return 0;
        }
        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    panic!(
                        "Invalid grapheme_idx passed to grapheme_idx_to_byte_idx: {grapheme_idx:?}"
                    );
                }
                #[cfg(not(debug_assertions))]
                {
                    0
                }
            },
            |fragment| fragment.byte_idx,
        )
    }
    // endregion
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl std::fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.string)
    }
}

impl Deref for Line {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn forward() {
        let s = "Löwe 老虎 Léopard Gepardi";
        let line = Line::from(s);
        let grapheme_idx = line.search_forward("pard", 2);
        assert_eq!(grapheme_idx, Some(11));
    }

    #[test]
    fn backward() {
        let s = "Löwe 老虎 Léopard Gepardi";
        let line = Line::from(s);
        let grapheme_idx = line.search_backward("pard", 22);
        assert_eq!(grapheme_idx, Some(18));
    }
}
