use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use std::{
    fmt,
    ops::{Deref, Range},
};

type GraphemeIdx = usize;
type ByteIdx = usize;

#[derive(Debug)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(&self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

#[derive(Debug)]
struct TextFragment {
    byte_idx: ByteIdx,
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

#[derive(Default)]
pub struct Line {
    string: String,
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
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
            let fragment_end = fragment.rendered_width.saturating_add(current_pos);
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

    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    pub fn insert_char(&mut self, ch: char, at: GraphemeIdx) {
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.byte_idx, ch);
        } else {
            self.string.push(ch);
        }
        self.rebuild_fragments();
    }

    pub fn delete(&mut self, at: GraphemeIdx) {
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

    pub fn search_from(&self, query: &str, from: GraphemeIdx) -> Option<GraphemeIdx> {
        // skip empty line or search from right of the end
        if self.is_empty() || self.grapheme_count() <= from {
            return None;
        }

        let from_byte_idx = self.grapheme_idx_to_byte_idx(from);
        self.string
            .get(from_byte_idx..)
            .and_then(|substr| substr.find(query))
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx).saturating_add(from))
    }

    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        if let Some(fragment) = self.fragments.get(grapheme_idx) {
            fragment.byte_idx
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Invalid grapheme_idx passed to grapheme_idx_to_byte_idx: {grapheme_idx:?}");
            }
            #[cfg(not(debug_assertions))]
            {
                0
            }
        }
    }

    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> GraphemeIdx {
        for (i, fragment) in self.fragments.iter().enumerate() {
            if fragment.byte_idx >= byte_idx {
                return i;
            }
        }
        #[cfg(debug_assertions)]
        {
            panic!("Invalid byte_idx passed to byte_idx_to_grapheme_idx: {byte_idx:?}");
        }
        #[cfg(not(debug_assertions))]
        {
            0
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
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
    fn search_for_text() {
        let s = "Löwe 老虎 Léopard Gepardi";
        let line = Line::from(s);
        let grapheme_idx = line.search_from("pard", 2);
        assert_eq!(grapheme_idx, Some(11));
    }
}
