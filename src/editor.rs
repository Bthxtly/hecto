use std::{
    env,
    panic::{set_hook, take_hook},
};

use crossterm::event::{
    Event::{self, Key},
    KeyEvent, KeyEventKind, read,
};

mod command;
mod commandbar;
mod documentstatus;
mod line;
mod messagebar;
mod position;
mod size;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use command::{
    Command::{self, Edit, Move, System},
    System::{Dismiss, Quit, Resize, Save, Search, SearchNext},
};
use commandbar::CommandBar;
use messagebar::MessageBar;
use position::Position;
use size::Size;
use statusbar::StatusBar;
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 3;

#[derive(Debug, Default, PartialEq)]
enum PromptType {
    Search,
    Save,
    #[default]
    None,
}

impl PromptType {
    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    prompt_type: PromptType,
    terminal_size: Size,
    title: String,
    quit_times: u8,
}

impl Editor {
    pub fn new() -> Result<Self, std::io::Error> {
        // custom Panic Hook to execute terminate before the program ends
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.handle_resize_command(size);

        let args: Vec<String> = env::args().collect();
        if let Some(filename) = args.get(1) {
            editor.view.load(filename);
        }

        editor.refresh_status();
        editor.message_bar.update_message(
            "HELP: <C-f> = find | <C-n> = search next | <C-s> = Save | <C-t> = Quit",
        );

        Ok(editor)
    }

    fn refresh_status(&mut self) {
        let status = self.view.get_status();

        let title = format!("{} - {NAME}", &status.filename);
        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }

        self.status_bar.update_status(status);
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => {
                    self.evaluate_event(event);
                }
                Err(err) => {
                    // panic if something goes wrong in a Release build
                    // in case user can not leave hecto with `CTRL-T`
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }

            self.refresh_status();
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }

        let _ = Terminal::hide_caret();

        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        if self.no_prompt() {
            self.message_bar.render(bottom_bar_row);
        } else {
            self.command_bar.render(bottom_bar_row);
        }

        let height = self.terminal_size.height;
        if height > 1 {
            self.status_bar.render(height.saturating_sub(2));
        }
        if height > 2 {
            self.view.render(0);
        }

        let new_caret_pos = if self.in_prompt() {
            Position {
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };

        let _ = Terminal::move_caret_to(&new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command);
            }
        }
    }

    fn process_command(&mut self, command: Command) {
        if let System(Resize(size)) = command {
            self.handle_resize_command(size);
        }

        match self.prompt_type {
            PromptType::None => self.process_command_no_prompt(command),
            PromptType::Save => self.process_command_during_save(command),
            PromptType::Search => self.process_command_during_search(command),
        }
    }

    fn handle_resize_command(&mut self, size: Size) {
        self.terminal_size = size;
        let bar_size = Size {
            height: 1,
            width: size.width,
        };

        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        self.status_bar.resize(bar_size);
        self.message_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
    }

    fn process_command_no_prompt(&mut self, command: Command) {
        if matches!(command, System(Quit)) {
            self.handle_quit();
            return;
        }
        self.reset_quit_times();

        match command {
            System(Quit | Resize(_) | Dismiss) => {}
            System(Save) => self.handle_save(),
            System(Search) => self.handle_search(),
            System(SearchNext) => self.handle_search_next(),
            Move(command) => self.view.handle_move_command(&command),
            Edit(command) => self.view.handle_edit_command(&command),
        }
    }

    fn reset_quit_times(&mut self) {
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.update_message("");
        }
    }

    // clippy::arithmetic_side_effects: quit_times is guaranteed to be between 0 and QUIT_TIMES
    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit(&mut self) {
        let is_modified = self.view.get_status().is_modified;
        if !is_modified || self.quit_times.saturating_add(1) == QUIT_TIMES {
            self.should_quit = true;
        } else if is_modified {
            self.update_message(&format!(
                "WARNING!!! File has unsaved changes. Press Ctrl-T {} more times to quit.",
                QUIT_TIMES - self.quit_times - 1
            ));
            self.quit_times += 1;
        }
    }

    fn handle_save(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    fn save(&mut self, filename: Option<&str>) {
        let result = if let Some(filename) = filename {
            self.view.save_as(filename)
        } else {
            self.view.save()
        };

        let msg = match result {
            Ok(()) => "File saved successfully",
            Err(_) => "Error writing file!",
        };
        self.update_message(msg);
    }

    fn handle_search(&mut self) {
        self.set_prompt(PromptType::Search);
        self.view.enter_search();
        self.update_message("");
    }

    fn handle_search_next(&mut self) {
        let success = self.view.search_next();
        if !success {
            self.update_message("Have no search query, please search for something first");
        }
    }

    fn process_command_during_save(&mut self, command: Command) {
        match command {
            System(Quit | Resize(_) | Save | Search | SearchNext) => {}
            System(Dismiss) => {
                self.dismiss_prompt();
                self.update_message("Save aborted");
            }
            Move(command) => self.command_bar.handle_move_command(&command),
            Edit(command) => {
                if matches!(command, command::Edit::InsertNewline) {
                    let pat = self.command_bar.value();
                    self.save(Some(&pat));
                    self.dismiss_prompt();
                } else {
                    self.command_bar.handle_edit_command(&command);
                }
            }
        }
    }

    fn process_command_during_search(&mut self, command: Command) {
        match command {
            System(Quit | Resize(_) | Save | Search | SearchNext) => {}
            Move(command) => self.command_bar.handle_move_command(&command),
            System(Dismiss) => {
                self.dismiss_prompt();
                self.view.dismiss_search();
                self.update_message("Search aborted");
            }
            Edit(command::Edit::InsertNewline) => {
                self.dismiss_prompt();
                self.view.exit_search();
            }
            Edit(command) => {
                self.command_bar.handle_edit_command(&command);
                let query = self.command_bar.value();
                self.view.search(&query);
            }
        }
    }

    fn update_message(&mut self, new_message: &str) {
        self.message_bar.update_message(new_message);
    }

    fn no_prompt(&self) -> bool {
        self.prompt_type.is_none()
    }

    fn in_prompt(&self) -> bool {
        !self.no_prompt()
    }

    fn set_prompt(&mut self, prompt_type: PromptType) {
        match prompt_type {
            PromptType::None => self.message_bar.set_needs_redraw(true),
            PromptType::Save => self.command_bar.set_prompt("Save as: "),
            PromptType::Search => self.command_bar.set_prompt("Search: "),
        }
        self.command_bar.clear_value();
        self.prompt_type = prompt_type;
    }

    fn dismiss_prompt(&mut self) {
        self.prompt_type = PromptType::None;
        self.message_bar.set_needs_redraw(true);
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye.\r\n");
        }
    }
}
