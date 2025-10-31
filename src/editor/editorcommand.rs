use crate::editor::KeyEvent;
use crate::editor::terminal::Size;
use crossterm::event::{Event, KeyCode, KeyModifiers};

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Up, _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (KeyCode::Home, _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::End, _) => Ok(Self::Move(Direction::End)),

                (KeyCode::Char('h'), _) => Ok(Self::Move(Direction::Left)),
                (KeyCode::Char('j'), _) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Char('k'), _) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Char('l'), _) => Ok(Self::Move(Direction::Right)),
                (KeyCode::Char('^'), _) => Ok(Self::Move(Direction::Home)),
                (KeyCode::Char('$'), _) => Ok(Self::Move(Direction::End)),
                (KeyCode::Char('n'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Down)),
                (KeyCode::Char('p'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Up)),
                (KeyCode::Char('f'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::PageDown)),
                (KeyCode::Char('b'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::PageUp)),
                _ => Err(format!("Key Code not supported: {code:?}")),
            },
            Event::Resize(width, height) => {
                let (height, width) = (height as usize, width as usize);
                Ok(Self::Resize(Size { height, width }))
            }
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}
