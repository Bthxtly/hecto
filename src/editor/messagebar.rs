use crate::editor::terminal::Terminal;

use super::terminal::Size;

pub struct MessageBar {
    current_message: String,
    needs_redraw: bool,
}

impl MessageBar {
    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }

        let line = self
            .current_message
            .get(..self.width)
            .unwrap_or(&self.current_message);

        let result = Terminal::print_row(self.position_y, &line);
        // will ignore this in release build
        debug_assert!(result.is_ok(), "Failed to render line");

        self.needs_redraw = false;
    }

    pub fn update_message(&mut self, new_message: String) {
        if new_message != self.current_message {
            self.current_message = new_message;
            self.needs_redraw = true;
        }
    }

    }
}

impl Default for MessageBar {
    fn default() -> Self {
        let size = Terminal::size().unwrap_or_default();
        let mut message_bar = Self {
            current_message: String::new(),
            width: size.width,
            position_y: 0,
            needs_redraw: true,
        };
        message_bar.resize(size);

        message_bar
    }
}
