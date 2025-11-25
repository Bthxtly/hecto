use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use std::{fmt, ops::Range};

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
    byte_index: usize,
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
        let grapheme_to_fragment = |(byte_index, grapheme): (usize, &str)| {
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
                byte_index,
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

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
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

    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    pub fn insert_char(&mut self, ch: char, at: usize) {
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.byte_index, ch);
        } else {
            self.string.push(ch);
        }
        self.rebuild_fragments();
    }

    pub fn delete(&mut self, at: usize) {
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.byte_index;
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

    pub fn split(&mut self, at: usize) -> Self {
        Self {
            string: self.string.split_off(at),
            fragments: self.fragments.split_off(at),
        }
    }

    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    pub fn search(&self, pat: &str) -> Option<usize> {
        self.string.find(pat)
    }

    fn index_to_location(&self, index: usize) -> usize {
        for (i, fragment) in self.fragments.iter().enumerate() {
            if fragment.byte_index == index {
                return i;
            }
        }
        unreachable!()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn search_for_text() {
        let s = "Löwe 老虎 Léopard Gepardi";
        let line = Line::from(s);
        let index = line.search("pard");
        assert_eq!(index, Some(17));
        let location = line.index_to_location(index.unwrap());
        assert_eq!(location, 11);
    }
}
