use std::cmp::min;

use super::super::{
    Size,
    command::{Edit, Move},
    line::Line,
    terminal::Terminal,
};

use super::UIComponent;

#[derive(Default)]
pub struct CommandBar {
    prompt: String,
    value: Line,
    needs_redraw: bool,
    size: Size,
}
impl CommandBar {
    pub fn handle_edit_command(&mut self, edit_command: &Edit) {
        match edit_command {
            Edit::InsertNewline | Edit::Delete => {}
            Edit::Insert(ch) => self.value.append_char(*ch),
            Edit::InsertTab => self.value.append_char('\t'),
            Edit::DeleteBackward => self.value.delete_last(),
        }
        self.set_needs_redraw(true);
    }

    pub fn handle_move_command(&self, _move_command: &Move) {
        todo!("ignore caret movement at this time");
    }

    pub fn value(&self) -> String {
        self.value.to_string()
    }

    pub fn caret_position_col(&self) -> usize {
        let characters_width = self
            .prompt
            .len()
            .saturating_add(self.value.grapheme_count());

        min(characters_width, self.size.width)
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
        self.set_needs_redraw(true);
    }

    pub fn clear_value(&mut self) {
        self.value = Line::default();
        self.set_needs_redraw(true);
    }
}

impl UIComponent for CommandBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_row: usize) -> Result<(), std::io::Error> {
        let area_for_value = self.size.width.saturating_sub(self.prompt.len());
        let value_end = self.value.width();
        let value_start = value_end.saturating_sub(area_for_value);
        let value_visible = self.value.get_visible_graphemes(value_start..value_end);

        let message = &format!("{}{}", self.prompt, value_visible);

        // FIXME: wish the editor is not too narrow 🙏
        assert!(message.len() < self.size.width);
        Terminal::print_row(origin_row, message)?;
        Ok(())
    }
}
