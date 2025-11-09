use std::fmt::Write as _;

use super::{
    DocumentStatus,
    terminal::{Size, Terminal},
};

pub struct StatusBar {
    current_status: DocumentStatus,
    needs_redraw: bool,
    bottom_margin: usize,
    width: usize,
    position_y: usize,
}

impl StatusBar {
    pub fn new(bottom_margin: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        Self {
            current_status: DocumentStatus::default(),
            needs_redraw: true,
            bottom_margin: 1,
            width: size.width,
            position_y: size.height.saturating_sub(bottom_margin).saturating_sub(1),
        }
    }

    pub fn render(&mut self) {
        let DocumentStatus {
            total_lines,
            current_line_index,
            is_modified,
            filename,
        } = &self.current_status;

        // left
        let mut line_text_left = String::new();
        if let Some(filename) = filename {
            let _ = write!(line_text_left, "{filename} ");
        }

        if *is_modified {
            line_text_left.push_str("[+] ");
        }

        // right
        let line_text_right = format!(
            "Ln {} of {}",
            current_line_index.saturating_add(1),
            total_lines
        );

        // cat
        let left_len = line_text_left.len();
        let right_len = line_text_right.len();
        let mut line_text = String::new();

        if left_len.saturating_add(right_len) > self.width {
            line_text_left.truncate(self.width);
            line_text.push_str(&line_text_left);
        } else {
            #[allow(clippy::arithmetic_side_effects)]
            let padding = self.width - line_text_left.len() - line_text_right.len();

            let _ = write!(
                line_text,
                "{}{}{}",
                line_text_left,
                " ".repeat(padding),
                line_text_right
            );
        }

        let result = Terminal::print_row(self.position_y, &line_text);
        // will ignore this in release build
        debug_assert!(result.is_ok(), "Failed to render line");

        self.needs_redraw = false;
    }

    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.current_status != status {
            self.current_status = status;
            self.needs_redraw = true;
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        self.position_y = size
            .height
            .saturating_sub(self.bottom_margin)
            .saturating_sub(1);
        self.needs_redraw = true;
    }
}
