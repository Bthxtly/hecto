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
        }
    }

    pub fn delete(&mut self, at: &Location) {
        let height = self.height();
        if let Some(line) = self.lines.get(at.line_index) {
            if at.line_index < height.saturating_sub(1)
                && at.grapheme_index == line.grapheme_count()
            {
                // join with the line below if at the end of line and there's line below
                let next_line = self.lines.remove(at.line_index.saturating_add(1));
                self.lines[at.line_index].append(&next_line);
            } else if at.line_index < height {
                // not at the end of the buffer
                self.lines[at.line_index].delete(at.grapheme_index);
            }
        }
    }

    pub fn insert_newline(&mut self, at: &Location) {
        if let Some(line) = self.lines.get_mut(at.line_index) {
            let new_line = line.split(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), new_line);
        } else {
            // add a new line if at the bottom of the document
            self.lines.push(Line::default());
        }
    }
}
