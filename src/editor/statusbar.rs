use std::fmt::Write as _;

use super::{
    documentstatus::DocumentStatus,
    terminal::{Size, Terminal},
};

pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    bottom_margin: usize,
    width: usize,
    position_y: usize,
    is_visible: bool,
}

impl StatusBar {
    pub fn new(bottom_margin: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        let mut status_bar = Self {
            current_status: DocumentStatus::default(),
            needs_redraw: true,
            bottom_margin: 1,
            width: size.width,
            position_y: 0,
            is_visible: false,
        };
        status_bar.resize(size);

        status_bar
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || !self.is_visible {
            return;
        }

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

            let result = Terminal::print_inverted_row(self.position_y, &to_print);
            // will ignore this in release build
            debug_assert!(result.is_ok(), "Failed to render line");

            self.needs_redraw = false;
        }
    }

    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.current_status != status {
            self.current_status = status;
            self.needs_redraw = true;
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;

        let mut position_y = 0;
        let mut is_visible = false;
        if let Some(result) = size
            .height
            .checked_sub(self.bottom_margin)
            .and_then(|result| result.checked_sub(1))
        {
            position_y = result;
            is_visible = true;
        }

        self.position_y = position_y;
        self.is_visible = is_visible;
        self.needs_redraw = true;
    }
}
