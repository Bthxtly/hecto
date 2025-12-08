use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
