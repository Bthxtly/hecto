use std::{
    env,
    panic::{set_hook, take_hook},
};

use commandbar::CommandBar;
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
    System::{Quit, Resize, Save},
};
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

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: Option<CommandBar>,
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
        editor.resize(size);

        let args: Vec<String> = env::args().collect();
        if let Some(filename) = args.get(1) {
            editor.view.load(filename);
        }

        editor.refresh_status();
        editor
            .message_bar
            .update_message("HELP: Ctrl-S = Save | Ctrl-T = Quit");

        Ok(editor)
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        self.message_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        if let Some(command_bar) = &mut self.command_bar {
            command_bar.resize(Size {
                height: 1,
                width: size.width,
            });
        }
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

            let status = self.view.get_status();
            self.status_bar.update_status(status);
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }

        let _ = Terminal::hide_caret();

        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        if let Some(command_bar) = &mut self.command_bar {
            command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }

        let height = self.terminal_size.height;
        if height > 1 {
            self.status_bar.render(height.saturating_sub(2));
        }
        if height > 2 {
            self.view.render(0);
        }

        let new_caret_pos = if let Some(comband_bar) = &self.command_bar {
            Position {
                row: bottom_bar_row,
                col: comband_bar.caret_position_col(),
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
        match command {
            System(Quit) => {
                if self.command_bar.is_none() {
                    // treat "Quit" only when outside of the prompt
                    self.handle_quit();
                }
            }
            System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(), // reset quit times for all other commands
        }
        match command {
            System(Quit | Resize(_)) => {} // already handled above
            System(Save) => {
                if self.command_bar.is_none() {
                    // treat "Save" only when outside of the prompt
                    self.handle_save();
                }
            }
            System(command::System::Dismiss) => {
                if self.command_bar.is_some() {
                    self.dismiss_prompt();
                    self.message_bar.update_message("Save aborted.");
                }
            }
            Edit(edit_command) => {
                if let Some(command_bar) = &mut self.command_bar {
                    if matches!(edit_command, command::Edit::InsertNewline) {
                        // take the file name after read a "Enter"
                        let filename = command_bar.value();
                        self.dismiss_prompt();
                        self.save(Some(&filename));
                    } else {
                        command_bar.handle_edit_command(&edit_command);
                    }
                } else {
                    self.view.handle_edit_command(&edit_command);
                }
            }
            Move(move_command) => {
                if let Some(command_bar) = &mut self.command_bar {
                    command_bar.handle_move_command(&move_command);
                } else {
                    self.view.handle_move_command(&move_command);
                }
            }
        }
    }

    fn reset_quit_times(&mut self) {
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.message_bar.update_message("");
        }
    }

    // clippy::arithmetic_side_effects: quit_times is guaranteed to be between 0 and QUIT_TIMES
    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit(&mut self) {
        let is_modified = self.view.get_status().is_modified;
        if !is_modified || self.quit_times.saturating_add(1) == QUIT_TIMES {
            self.should_quit = true;
        } else if is_modified {
            self.message_bar.update_message(&format!(
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
            self.show_prompt();
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
        self.message_bar.update_message(msg);
    }

    fn show_prompt(&mut self) {
        let mut command_bar = CommandBar::default();
        command_bar.set_prompt("Save as: ");
        command_bar.resize(Size {
            height: 1,
            width: self.terminal_size.width,
        });
        self.command_bar = Some(command_bar);
    }

    fn dismiss_prompt(&mut self) {
        self.command_bar = None;
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
