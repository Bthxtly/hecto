use std::{
    env,
    panic::{set_hook, take_hook},
};

use command::{
    Command::{self, Edit, Move, System},
    System::{Quit, Resize, Save},
};
use crossterm::event::{
    Event::{self, Key},
    KeyEvent, KeyEventKind, read,
};

mod command;
mod documentstatus;
mod fileinfo;
mod messagebar;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use messagebar::MessageBar;
use statusbar::StatusBar;
use terminal::{Size, Terminal};
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
            System(Quit) => self.handle_quit(),
            System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(), // reset quit times for all other commands
        }
        match command {
            System(Quit | Resize(_)) => {} // already handled above
            System(Save) => self.handle_save(),
            Edit(edit_command) => self.view.handle_edit_command(&edit_command),
            Move(move_command) => self.view.handle_move_command(&move_command),
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
        let msg = match self.view.save() {
            Ok(()) => "File saved successfully",
            Err(_) => "Error writing file!",
        };
        self.message_bar.update_message(msg);
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }

        let _ = Terminal::hide_caret();

        let height = self.terminal_size.height;
        self.message_bar.render(height.saturating_sub(1));
        if height > 1 {
            self.status_bar.render(height.saturating_sub(2));
        }
        if height > 2 {
            self.view.render(0);
        }

        let _ = Terminal::move_caret_to(&self.view.caret_position());
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
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
