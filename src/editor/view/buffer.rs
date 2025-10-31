use std::fs::read_to_string;

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
}
