use std::{
    cmp::min,
    env,
    panic::{set_hook, take_hook},
};

use crossterm::event::{
    Event::{self, Key, Resize},
    KeyCode::{self},
    KeyEvent, KeyEventKind, KeyModifiers, read,
};

mod terminal;
use terminal::{Position, Size, Terminal};

mod view;
use view::View;

#[derive(Default)]
struct Location {
    x: usize,
    y: usize,
}

pub struct Editor {
    should_quit: bool,
    location: Location,
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
            location: Location::default(),
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

    fn move_point(&mut self, key_code: KeyCode) {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::size().unwrap_or_default();
        match key_code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = min(height.saturating_sub(1), y.saturating_add(1));
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Right => {
                x = min(width.saturating_sub(1), x.saturating_add(1));
            }
            KeyCode::PageUp => {
                y = 0;
            }
            KeyCode::PageDown => {
                y = height.saturating_sub(1);
            }
            KeyCode::Home => {
                x = 0;
            }
            KeyCode::End => {
                x = width.saturating_sub(1);
            }
            _ => (),
        }
        self.location = Location { x, y };
    }

    fn evaluate_event(&mut self, event: Event) -> Result<(), std::io::Error> {
        match event {
            Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                    // CTRL+t to quit
                    self.should_quit = true;
                }
                (
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Home
                    | KeyCode::End,
                    _,
                ) => {
                    self.move_point(code);
                }
                _ => (),
            },

            Resize(width, height) => {
                let (height, width) = (height as usize, width as usize);
                self.view.resize(Size { height, width });
            }
            _ => (),
        }
        Ok(())
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();

        self.view.render();
        let _ = Terminal::move_caret_to(&Position {
            col: self.location.x,
            row: self.location.y,
        });

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
