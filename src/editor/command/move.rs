use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
