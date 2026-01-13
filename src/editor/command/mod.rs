use super::size::Size;
use crossterm::event::Event;
pub use edit::Edit;
pub use r#move::Move;
pub use system::System;

mod edit;
mod r#move;
mod system;

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
