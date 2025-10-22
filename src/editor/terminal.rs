use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::io::stdout;

pub struct Terminal {}

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(0, 0)?;
        Self::execute()?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    pub fn move_cursor_to(x: u16, y: u16) -> Result<(), std::io::Error> {
        queue!(stdout(), MoveTo(p.x, p.y))?;
        Ok(())
    }

    pub fn size() -> Result<(u16, u16), std::io::Error> {
        size()
    pub fn print(s: &str) -> Result<(), std::io::Error> {
        queue!(stdout(), Print(s))?;
        Ok(())
    }

    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()?;
        Ok(())
    }
}
