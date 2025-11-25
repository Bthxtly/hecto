use std::fs::File;
use std::fs::read_to_string;
use std::io::Write;

use super::Location;
use super::fileinfo::FileInfo;
use crate::editor::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub file_info: FileInfo,
    pub lines: Vec<Line>,
    pub dirty: bool,
}

impl Buffer {
    pub fn load(filename: &str) -> Self {
        if let Ok(string) = read_to_string(filename) {
            let lines = string.lines().map(Line::from).collect();
            Self {
                file_info: FileInfo::from(filename),
                lines,
                dirty: false,
            }
        } else {
            // open as an empty file if file doesn't exist
            Self {
                file_info: FileInfo::from(filename),
                lines: vec![Line::default()],
                dirty: true,
            }
        }
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let file_info = FileInfo::from(filename);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;
        Ok(())
    }

    fn save_to_file(&self, file_info: &FileInfo) -> Result<(), std::io::Error> {
        if let Some(path) = file_info.get_path() {
            let mut file = File::create(path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }

        Ok(())
    }

    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
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
        self.dirty = true;
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
            self.dirty = true;
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
        self.dirty = true;
    }

    pub fn search(&self, pat: &str) {
        for line in &self.lines {
            line.search(pat);
        }
    }
}
