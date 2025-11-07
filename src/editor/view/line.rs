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
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments = Self::str_to_fragments(line_str);
        Self { fragments }
    }

    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        let grapheme_to_fragment = |grapheme: &str| {
            let (replacement, rendered_width) = Self::replacement_character(grapheme).map_or_else(
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
                grapheme: grapheme.to_string(),
                rendered_width,
                replacement,
            }
        };

        line_str.graphemes(true).map(grapheme_to_fragment).collect()
    }

    fn replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();
        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if for_str.chars().all(|char| char.is_control()) => Some('▯'),
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

    pub fn insert_char(&mut self, ch: char, grapheme_index: usize) {
        let mut new_line = String::new();

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                new_line.push(ch);
            }
            new_line.push_str(&fragment.grapheme);
        }

        // insert at the end of line
        if grapheme_index >= self.fragments.len() {
            new_line.push(ch);
        }

        self.fragments = Self::str_to_fragments(&new_line);
    }

    pub fn delete(&mut self, grapheme_index: usize) {
        let mut new_line = String::new();

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                continue;
            }
            new_line.push_str(&fragment.grapheme);
        }

        self.fragments = Self::str_to_fragments(&new_line);
    }

    pub fn append(&mut self, other: &Self) {
        let mut concat = self.to_string();
        concat.push_str(&other.to_string());
        self.fragments = Self::str_to_fragments(&concat)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = self
            .fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect();

        write!(f, "{result}")
    }
}
