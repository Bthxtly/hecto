use crate::editor::KeyEvent;
use crate::editor::Size;
use crossterm::event::{Event, KeyCode, KeyModifiers};

pub enum Move {
    PageUp,
    PageDown,
    StartOfLine,
    EndOfLine,
    Up,
    Left,
    Right,
    Down,
}

impl TryFrom<KeyEvent> for Move {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        if modifiers == KeyModifiers::NONE {
            match code {
                KeyCode::Up => Ok(Move::Up),
                KeyCode::Down => Ok(Move::Down),
                KeyCode::Left => Ok(Move::Left),
                KeyCode::Right => Ok(Move::Right),
                KeyCode::PageDown => Ok(Move::PageDown),
                KeyCode::PageUp => Ok(Move::PageUp),
                KeyCode::Home => Ok(Move::StartOfLine),
                KeyCode::End => Ok(Move::EndOfLine),
                _ => Err(format!("Unsupported code: {code:?}")),
            }
        } else {
            Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ))
        }
    }
}

pub enum Edit {
    Insert(char),
    InsertTab,
    InsertNewline,
    Delete,
    DeleteBackward,
}

impl TryFrom<KeyEvent> for Edit {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        match (code, modifiers) {
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(ch)),
            (KeyCode::Tab, KeyModifiers::NONE) => Ok(Self::InsertTab),
            (KeyCode::Enter, KeyModifiers::NONE) => Ok(Self::InsertNewline),
            (KeyCode::Delete, KeyModifiers::NONE) => Ok(Self::Delete),
            (KeyCode::Backspace, KeyModifiers::NONE) => Ok(Self::DeleteBackward),
            _ => Err(format!(
                "Unsupported code: {code:?} with modifiers {modifiers:?}"
            )),
        }
    }
}

pub enum System {
    Save,
    Search,
    Dismiss,
    Resize(Size),
    Quit,
}

impl TryFrom<KeyEvent> for System {
    type Error = String;

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        if modifiers == KeyModifiers::CONTROL {
            match code {
                KeyCode::Char('t') => Ok(Self::Quit),
                KeyCode::Char('s') => Ok(Self::Save),
                KeyCode::Char('f') => Ok(Self::Search),
                _ => Err(format!("Unknown not CONTROL+{code:?} combination")),
            }
        } else if modifiers == KeyModifiers::NONE && matches!(code, KeyCode::Esc) {
            Ok(Self::Dismiss)
        } else {
            Err(format!(
                "Unsupported code: {code:?} with modifiers {modifiers:?}"
            ))
        }
    }
}

pub enum Command {
    Move(Move),
    Edit(Edit),
    System(System),
}

// clippy::as_conversions: Will run into problems for rare edge case systems where usize < u16
#[allow(clippy::as_conversions)]
impl TryFrom<Event> for Command {
    type Error = String;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(key_event) => Edit::try_from(key_event)
                .map(Command::Edit)
                .or_else(|_| Move::try_from(key_event).map(Command::Move))
                .or_else(|_| System::try_from(key_event).map(Command::System))
                .map_err(|_| format!("Event not supported: {key_event:?}")),
            Event::Resize(width, height) => Ok(Self::System(System::Resize(Size {
                height: height as usize,
                width: width as usize,
            }))),
            _ => Err(format!("Event not supported: {event:?}")),
        }
    }
}
