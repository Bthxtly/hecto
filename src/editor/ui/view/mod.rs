use super::super::{
    NAME, Position, Size, VERSION,
    command::{Edit, Move},
    documentstatus::DocumentStatus,
    line::Line,
    position::{Col, Row},
    terminal::Terminal,
};
use super::UIComponent;
use buffer::Buffer;
use location::Location;
use search_direction::SearchDirection;
use searchinfo::SearchInfo;
use std::cmp::{max, min};

mod buffer;
mod fileinfo;
mod location;
mod search_direction;
mod searchinfo;

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

    // region: save
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.buffer.save()
    }

    pub fn save_as(&mut self, filename: &str) -> Result<(), std::io::Error> {
        self.buffer.save_as(filename)
    }
    // endregion

    // region: search
    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            previous_location: self.text_location,
            query: None,
        });
    }

    pub fn dismiss_search(&mut self) {
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.previous_location;
            self.search_info = None;
            // ensure the previous location is still visible even if the terminal has been resized during search
            self.scroll_text_location_into_view();
        }
    }

    pub fn search(&mut self, query: &str) {
        if let Some(search_info) = &mut self.search_info {
            search_info.query = Some(Line::from(query));
        }
        self.search_in_direction(self.text_location, SearchDirection::default());
    }

    // Attempts to get the current search query - for scenarios where the search query absolutely must be there.
    // Panics if not present in debug, or if search info is not present in debug
    // Returns None on release.
    fn get_search_query(&self) -> Option<&Line> {
        let query = self
            .search_info
            .as_ref()
            .and_then(|search_info| search_info.query.as_ref());
        debug_assert!(
            query.is_some(),
            "Attempting to search with malformed searchinfo present"
        );
        query
    }

    fn search_in_direction(&mut self, from: Location, direction: SearchDirection) {
        if let Some(location) = self.get_search_query().and_then(|query| {
            if query.is_empty() {
                None
            } else if direction == SearchDirection::Forward {
                self.buffer.search_forward(query, &from)
            } else if direction == SearchDirection::Backwoard {
                self.buffer.search_backward(query, &from)
            } else {
                unreachable!()
            }
        }) {
            self.text_location = location;
            self.scroll_text_location_into_view();
        }
        self.set_needs_redraw(true);
    }

    pub fn search_next(&mut self) {
        let step_right = self
            .get_search_query()
            .map_or(1, |query| max(query.grapheme_count(), 1));
        let location = Location {
            line_idx: self.text_location.line_idx,
            grapheme_idx: self.text_location.grapheme_idx.saturating_add(step_right),
        };
        self.search_in_direction(location, SearchDirection::Forward);
    }

    pub fn search_backward(&mut self) {
        self.search_in_direction(self.text_location, SearchDirection::Backwoard);
    }
    // endregion

    pub fn get_status(&self) -> DocumentStatus {
        DocumentStatus {
            total_lines: self.buffer.get_height(),
            current_line_idx: self.text_location.line_idx,
            is_modified: self.buffer.dirty,
            filename: format!("{}", self.buffer.file_info),
        }
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(&self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_idx;
        let col = self
            .buffer
            .lines
            .get(row)
            .map_or(0, |line| line.width_until(self.text_location.grapheme_idx));

        Position { row, col }
    }

    // region: edit
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
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);

        self.buffer.insert_char(ch, &self.text_location);

        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
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
        if self.text_location.line_idx == 0 && self.text_location.grapheme_idx == 0 {
            return;
        }
        self.handle_move_command(&Move::Left);
        self.delete();
    }
    // endregion

    // region: move
    pub fn handle_move_command(&mut self, command: &Move) {
        let Size { height, .. } = self.size;

        // This match moves the position, but does not check for all boundaries.
        // The final boundary checking happens after the match statement.
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
        let line_idx = &mut self.text_location.line_idx;
        *line_idx = line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        let line_idx = &mut self.text_location.line_idx;
        *line_idx = line_idx.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self, step: usize) {
        let grapheme_idx = &mut self.text_location.grapheme_idx;

        if *grapheme_idx == 0 && self.text_location.line_idx > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        } else {
            *grapheme_idx = grapheme_idx.saturating_sub(step);
            self.snap_to_valid_grapheme();
        }
    }

    fn move_right(&mut self, step: usize) {
        let grapheme_idx = &mut self.text_location.grapheme_idx;
        let length = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);

        *grapheme_idx = grapheme_idx.saturating_add(step);

        if *grapheme_idx > length {
            self.move_down(1);
        } else {
            self.snap_to_valid_grapheme();
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_idx = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);
    }

    // ensure self.location.grapheme_idx points to a valid grapheme idx by snapping it
    // to the left most grapheme if appropriate
    // do not trigger scolling
    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, |line| {
                min(line.grapheme_count(), self.text_location.grapheme_idx)
            });
    }

    // ensure self.location.grapheme_idx points to a valid grapheme idx by snapping it
    // to the left most grapheme if appropriate
    // do not trigger scolling
    // line_idx can be exactly self.buffer.height() since sometimes we want to modify below buffer
    fn snap_to_valid_line(&mut self) {
        self.text_location.line_idx = min(self.text_location.line_idx, self.buffer.get_height());
    }

    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    fn scroll_vertically(&mut self, to: Row) {
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

        self.set_needs_redraw(offset_changed || self.get_needs_redraw());
    }

    fn scroll_horizontally(&mut self, to: Col) {
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

        self.set_needs_redraw(offset_changed || self.get_needs_redraw());
    }
    // endregion

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

    fn get_needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }

    fn draw(&mut self, origin_row: usize) -> Result<(), std::io::Error> {
        let Size { height, width } = self.size;
        let end_y = origin_row.saturating_add(height);

        let top_third = height.div_ceil(3); // a good position to put our welcome message
        let scroll_top = self.scroll_offset.row;

        for current_row in origin_row..end_y {
            // to get the correct line idx, we have to take current_row (the absolute row on
            // screen), subtract origin_row to get the current row relative to the view (ranging from
            // 0 to self.size.height) and add the scroll offset
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);
            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                let query = self
                    .search_info
                    .as_ref()
                    .and_then(|search_info| search_info.query.as_deref());
                let selected_match = (self.text_location.line_idx == line_idx && query.is_some())
                    .then_some(self.text_location.grapheme_idx);
                Terminal::print_annotated_row(
                    current_row,
                    &line.get_annotated_visible_substr(left..right, query, selected_match),
                )?;
            } else if (current_row == top_third) && self.buffer.is_empty() {
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
