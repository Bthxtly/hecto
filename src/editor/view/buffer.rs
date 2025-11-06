use std::fs::read_to_string;

use super::Location;
use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            lines: read_to_string(filename)?.lines().map(Line::from).collect(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_char(&mut self, ch: char, at: &Location) {
        if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_char(ch, at.grapheme_index);
        } else {
            self.lines.push(Line::from(&ch.to_string()));
        };
    }
}
