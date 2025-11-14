use crate::editor::terminal::Terminal;

use super::{terminal::Size, uicomponent::UIComponent};

#[derive(Default)]
pub struct MessageBar {
    current_message: String,
    needs_redraw: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: String) {
        if new_message != self.current_message {
            self.current_message = new_message;
            self.needs_redraw = true;
        }
    }
}

impl UIComponent for MessageBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, _size: Size) {}

    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        Terminal::print_row(origin_y, &self.current_message)?;
        Ok(())
    }
}
