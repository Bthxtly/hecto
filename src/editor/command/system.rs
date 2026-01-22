use crate::editor::size::Size;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum System {
    Save,
    Search,
    SearchNext,
    SearchPrevious,
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
                KeyCode::Char('n') => Ok(Self::SearchNext),
                KeyCode::Char('p') => Ok(Self::SearchPrevious),
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
