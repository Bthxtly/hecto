use super::{
    documentstatus::DocumentStatus,
    terminal::{Size, Terminal},
    uicomponent::UIComponent,
};

#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    size: Size,
}

impl StatusBar {
    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.current_status != status {
            self.current_status = status;
            self.set_needs_redraw(true);
        }
    }
}

impl UIComponent for StatusBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_y: usize) -> Result<(), std::io::Error> {
        if let Ok(size) = Terminal::size() {
            // left
            let filename = &self.current_status.filename;
            let line_count = &self.current_status.line_count_to_string();
            let modified_indicator = &self.current_status.modified_indicator_to_string();
            let beginning = if modified_indicator.is_empty() {
                format!("{filename} - {line_count}")
            } else {
                format!("{filename} {modified_indicator} - {line_count}")
            };

            // right
            let position_indicator = &self.current_status.position_indicator_to_string();

            // cat
            let remainder_width = size.width.saturating_sub(beginning.len());
            let status = format!("{beginning}{position_indicator:>remainder_width$}");

            // Only print out the status if it fits.
            // Otherwise write out an empty string to ensure the row is cleared.
            let to_print = if status.len() <= size.width {
                status
            } else {
                String::new()
            };

            let result = Terminal::print_inverted_row(origin_y, &to_print);
            // will ignore this in release build
            debug_assert!(result.is_ok(), "Failed to render line");

            self.set_needs_redraw(false);
        }

        Ok(())
    }
}
