use unicode_segmentation::UnicodeSegmentation;

use std::cmp;
use std::ops::Range;

pub struct Line {
    graphemes: Vec<String>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Self {
            graphemes: line_str
                .graphemes(true)
                .map(|g| g.to_string())
                .collect::<Vec<String>>(),
        }
    }

    pub fn get(&self, range: Range<usize>) -> String {
        let start = range.start;
        let end = cmp::min(range.end, self.graphemes.len());

        self.graphemes.get(start..end).unwrap_or_default().concat()
    }

    pub fn len(&self) -> usize {
        self.graphemes.len()
    }
}
