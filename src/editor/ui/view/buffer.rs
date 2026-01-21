use super::Location;
use super::fileinfo::FileInfo;
use crate::editor::line::Line;
use std::fs::File;
use std::fs::read_to_string;
use std::io::Write;

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

    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn get_height(&self) -> usize {
        self.lines.len()
    }

    // region: save
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
    // endregion

    // region: edit
    pub fn insert_char(&mut self, ch: char, at: &Location) {
        if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert_char(ch, at.grapheme_idx);
        } else {
            self.lines.push(Line::from(&ch.to_string()));
        }
        self.dirty = true;
    }

    pub fn delete(&mut self, at: &Location) {
        let height = self.get_height();
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.line_idx < height.saturating_sub(1) && at.grapheme_idx == line.grapheme_count() {
                // join with the line below if at the end of line and there's line below
                let next_line = self.lines.remove(at.line_idx.saturating_add(1));
                self.lines[at.line_idx].append(&next_line);
            } else if at.line_idx < height {
                // not at the end of the buffer
                self.lines[at.line_idx].delete(at.grapheme_idx);
            }
            self.dirty = true;
        }
    }

    pub fn insert_newline(&mut self, at: &Location) {
        if let Some(line) = self.lines.get_mut(at.line_idx) {
            let new_line = line.split(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), new_line);
        } else {
            // add a new line if at the bottom of the document
            self.lines.push(Line::default());
        }
        self.dirty = true;
    }
    // endregion

    // region: search
    pub fn search_forward(&self, query: &str, from: &Location) -> Option<Location> {
        for (line_idx, line) in self.lines.iter().enumerate().skip(from.line_idx) {
            let from_grapheme_idx = if line_idx == from.line_idx {
                from.grapheme_idx
            } else {
                0
            };

            if let Some(grapheme_idx) = line.search_forward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        None
    }

    pub fn search_backward(&self, query: &str, from: &Location) -> Option<Location> {
        for (line_idx, line) in self.lines.iter().enumerate().skip(from.line_idx) {
            let from_grapheme_idx = if line_idx == from.line_idx {
                from.grapheme_idx
            } else {
                0
            };

            if let Some(grapheme_idx) = line.search_forward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        None
    }
    // endregion
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() -> Buffer {
        let mut buffer = Buffer::default();
        let string = concat!(
            "0_234567890\n",
            "foo345foo90\n",
            "2_234567890\n",
            "3_234567890\n",
            "4_2foo67890\n",
            "5_234567890\n",
            "6_234567foo\n",
            "7_234barfoo\n",
            "8_234567890\n",
            "9_234567890\n",
        );
        buffer.lines = string.lines().map(Line::from).collect();
        buffer
    }

    #[test]
    fn search_from_beginning() {
        let buffer = init();
        let from = Location {
            line_idx: 0,
            grapheme_idx: 0,
        };
        let found = Location {
            line_idx: 1,
            grapheme_idx: 0,
        };
        assert_eq!(buffer.search_forward("foo", &from), Some(found));
    }

    #[test]
    fn search_for_next() {
        let buffer = init();
        let step_right = 1;
        let from = Location {
            line_idx: 1,
            #[allow(clippy::identity_op)]
            grapheme_idx: 0 + step_right,
        };
        let found = Location {
            line_idx: 1,
            grapheme_idx: 6,
        };
        assert_eq!(buffer.search_forward("foo", &from), Some(found));
    }

    #[test]
    fn search_for_next_at_end() {
        let buffer = init();
        let step_right = 3;
        let from = Location {
            line_idx: 6,
            grapheme_idx: 8 + step_right,
        };
        let found = Location {
            line_idx: 7,
            grapheme_idx: 8,
        };
        assert_eq!(buffer.search_forward("foo", &from), Some(found));
    }

    #[test]
    fn search_from_middle() {
        let buffer = init();
        let from = Location {
            line_idx: 3,
            grapheme_idx: 9,
        };
        let found = Location {
            line_idx: 4,
            grapheme_idx: 3,
        };
        assert_eq!(buffer.search_forward("foo", &from), Some(found));
    }
}
