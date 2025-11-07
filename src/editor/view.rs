use std::cmp::min;

use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};

mod buffer;
use buffer::Buffer;

mod line;
use line::Line;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
}

impl View {
    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
        }
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(&self.scroll_offset)
            .into()
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });

        Position { row, col }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }

        let Size { height, width } = self.size;
        if height == 0 || width == 0 {
            return;
        }

        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit up or down
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.row;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                let truncated_line = line.get_visible_graphemes(left..right);
                Self::render_line(current_row, &truncated_line);
            } else if (current_row == vertical_center) && self.buffer.is_empty() {
                // render welcome message if no file is opened
                Self::render_line(current_row, &Self::build_welcome_message(width));
            } else {
                // else render tilde at empty lines
                Self::render_line(current_row, "~");
            }
        }

        self.needs_redraw = false;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);

        // will ignore this in release build
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    fn build_welcome_message(width: usize) -> String {
        let welcome_message = format!("{NAME} editor -- version {VERSION}");

        let len = welcome_message.len();
        if width <= len {
            return "~".to_string();
        }

        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit to the left or right.
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;

        let mut full_message = format!("~{}{}", " ".repeat(padding), welcome_message);
        full_message.truncate(width);

        full_message
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Insert(ch) => self.insert_char(ch),
            EditorCommand::Enter => self.insert_newline(),
            EditorCommand::Tab => self.insert_tab(),
            EditorCommand::Delete => self.delete(),
            EditorCommand::Backspace => self.delete_backward(),
            EditorCommand::Quit => {}
        }
    }

    fn resize(&mut self, size: Size) {
        self.size = size;
        self.needs_redraw = true;
    }

    fn move_text_location(&mut self, direction: Direction) {
        let Size { height, .. } = self.size;

        // This match moves the positon, but does not check for all boundaries.
        // The final boundarline checking happens after the match statement.
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(1),
            Direction::Right => self.move_right(1),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }

        self.scroll_text_location_into_view();
    }

    fn move_up(&mut self, step: usize) {
        let line_index = &mut self.text_location.line_index;
        *line_index = line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        let line_index = &mut self.text_location.line_index;
        *line_index = line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self, step: usize) {
        let grapheme_index = &mut self.text_location.grapheme_index;

        if *grapheme_index == 0 && self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        } else {
            *grapheme_index = grapheme_index.saturating_sub(step);
            self.snap_to_valid_grapheme();
        }
    }

    fn move_right(&mut self, step: usize) {
        let grapheme_index = &mut self.text_location.grapheme_index;
        let length = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        *grapheme_index = grapheme_index.saturating_add(step);

        if *grapheme_index > length {
            self.move_to_start_of_line();
            self.move_down(1);
        } else {
            self.snap_to_valid_grapheme();
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
    }

    // ensure self.location.grapheme_index points to a valid grapheme index by snapping it
    // to the left most grapheme if appropriate
    // do not trigger scolling
    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                min(line.grapheme_count(), self.text_location.grapheme_index)
            });
    }

    // ensure self.location.grapheme_index points to a valid grapheme index by snapping it
    // to the left most grapheme if appropriate
    // do not trigger scolling
    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(self.text_location.line_index, self.buffer.height());
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    fn scroll_vertically(&mut self, to: usize) {
        let Position { row, .. } = &mut self.scroll_offset;
        let Size { height, .. } = self.size;

        let offset_changed = if to < *row {
            *row = to;
            true
        } else if to >= row.saturating_add(height) {
            *row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn scroll_horizontally(&mut self, to: usize) {
        let Position { col, .. } = &mut self.scroll_offset;
        let Size { width, .. } = self.size;

        let offset_changed = if to < *col {
            *col = to;
            true
        } else if to >= col.saturating_add(width) {
            *col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };

        self.needs_redraw = self.needs_redraw || offset_changed;
    }

    fn insert_char(&mut self, ch: char) {
        let old_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        self.buffer.insert_char(ch, &self.text_location);

        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);

        if new_len.saturating_sub(old_len) > 0 {
            self.move_text_location(Direction::Right);
        }
        self.needs_redraw = true;
    }

    fn delete(&mut self) {
        self.buffer.delete(&self.text_location);
        self.needs_redraw = true;
    }

    fn delete_backward(&mut self) {
        // do nothing if at top-left corner
        if self.text_location.line_index == 0 && self.text_location.grapheme_index == 0 {
            return;
        }
        self.move_text_location(Direction::Left);
        self.delete();
    }

    fn insert_tab(&mut self) {
        self.insert_char('\t');
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(&self.text_location);
        self.move_text_location(Direction::Right);
        self.needs_redraw = true;
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }
}
