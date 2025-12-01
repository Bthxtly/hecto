use std::cmp::min;

use super::{
    NAME, Position, Size, VERSION,
    command::{Edit, Move},
    documentstatus::DocumentStatus,
    line::Line,
    terminal::Terminal,
    uicomponent::UIComponent,
};

use buffer::Buffer;

mod buffer;
mod fileinfo;

struct SearchInfo {
    previous_location: Location,
}

#[derive(Default, Clone, Copy)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

#[derive(Default)]
pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
    search_info: Option<SearchInfo>,
}

impl View {
    pub fn load(&mut self, filename: &str) {
        self.buffer = Buffer::load(filename);
    }

    pub fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), std::io::Error> {
        self.buffer.save_as(filename)
    }

    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            previous_location: self.text_location,
        });
    }

    pub fn exit_search(&mut self) {
        self.search_info = None;
    }

    pub fn dismiss_search(&mut self) {
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.previous_location;
        }
        self.search_info = None;
        self.scroll_text_location_into_view();
    }

    pub fn search(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        if let Some(location) = self.buffer.search(query) {
            self.text_location = location;
            self.scroll_text_location_into_view();
        }
    }

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.height(),
            current_line_index: self.text_location.line_index,
            is_modified: self.buffer.dirty,
            filename: format!("{}", self.buffer.file_info),
        }
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(&self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });

        Position { row, col }
    }

    pub fn handle_edit_command(&mut self, command: &Edit) {
        match command {
            Edit::Insert(ch) => self.insert_char(*ch),
            Edit::InsertTab => self.insert_tab(),
            Edit::InsertNewline => self.insert_newline(),
            Edit::Delete => self.delete(),
            Edit::DeleteBackward => self.delete_backward(),
        }
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
            self.handle_move_command(&Move::Right);
        }
        self.set_needs_redraw(true);
    }

    fn insert_tab(&mut self) {
        self.insert_char('\t');
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(&self.text_location);
        self.handle_move_command(&Move::Right);
        self.set_needs_redraw(true);
    }

    fn delete(&mut self) {
        self.buffer.delete(&self.text_location);
        self.set_needs_redraw(true);
    }

    fn delete_backward(&mut self) {
        // do nothing if at top-left corner
        if self.text_location.line_index == 0 && self.text_location.grapheme_index == 0 {
            return;
        }
        self.handle_move_command(&Move::Left);
        self.delete();
    }

    pub fn handle_move_command(&mut self, command: &Move) {
        let Size { height, .. } = self.size;

        // This match moves the positon, but does not check for all boundaries.
        // The final boundarline checking happens after the match statement.
        match command {
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::Left => self.move_left(1),
            Move::Right => self.move_right(1),
            Move::PageUp => self.move_up(height.saturating_sub(1)),
            Move::PageDown => self.move_down(height.saturating_sub(1)),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
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

        if !self.needs_redraw() {
            self.set_needs_redraw(offset_changed);
        }
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

        if !self.needs_redraw() {
            self.set_needs_redraw(offset_changed);
        }
    }

    fn render_line(at: usize, line_text: &str) -> Result<(), std::io::Error> {
        Terminal::print_row(at, line_text)?;
        Ok(())
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let welcome_message = format!("{NAME} editor -- version {VERSION}");

        let len = welcome_message.len();
        let remaining_width = width.saturating_sub(1);
        if remaining_width <= len {
            return "~".to_string();
        }

        format!("{:<1}{:^remaining_width$}", "~", welcome_message)
    }
}

impl UIComponent for View {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_row: usize) -> Result<(), std::io::Error> {
        let Size { height, width } = self.size;
        let end_y = origin_row.saturating_add(height);

        // we allow this since we don't care if our welcome message is put _exactly_ in the middle.
        // it's allowed to be a bit up or down
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let scroll_top = self.scroll_offset.row;

        for current_row in origin_row..end_y {
            // to get the correct line index, we have to take current_row (the absolute row on
            // screen), subtract origin_row to get the current row relative to the view (ranging from
            // 0 to self.size.height) and add the scroll offset
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);
            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                let truncated_line = &line.get_visible_graphemes(left..right);
                Self::render_line(current_row, truncated_line)?;
            } else if (current_row == vertical_center) && self.buffer.is_empty() {
                // render welcome message if no file is opened
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                // else render tilde at empty lines
                Self::render_line(current_row, "~")?;
            }
        }

        Ok(())
    }
}
