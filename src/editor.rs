use std::{
    env,
    panic::{set_hook, take_hook},
};

use crossterm::event::{
    Event::{self, Key},
    KeyEvent, KeyEventKind, read,
};

mod editorcommand;

mod terminal;
use editorcommand::EditorCommand;
use terminal::Terminal;

mod view;
use view::View;

pub struct Editor {
    should_quit: bool,
    view: View,
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

        let mut view = View::default();
        let args: Vec<String> = env::args().collect();
        if let Some(filename) = args.get(1) {
            view.load(filename);
        }

        Ok(Self {
            should_quit: false,
            view,
        })
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }

            match read() {
                Ok(event) => {
                    self.evaluate_event(event)?;
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
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: Event) -> Result<(), std::io::Error> {
        let should_process = match &event {
            Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            match EditorCommand::try_from(event) {
                Ok(command) => {
                    if matches!(command, EditorCommand::Quit) {
                        self.should_quit = true;
                    } else {
                        self.view.handle_command(command);
                    }
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not handle command: {err}")
                    };
                }
            }
        }
        // NOTE: this is too strict
        // else {
        //     #[cfg(debug_assertions)]
        //     {
        //         panic!("Received and discarded unsupported or non-press event.");
        //     }
        // }

        Ok(())
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();

        self.view.render();
        let _ = Terminal::move_caret_to(&self.view.get_position());

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
