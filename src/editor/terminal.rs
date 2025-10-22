use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size};
use std::io::{Write, stdout};

pub struct Terminal;

pub struct Size {
    pub height: u16,
    pub width: u16,
}

pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?;
        Ok(())
    }

    pub fn clear_screen() -> Result<(), std::io::Error> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), std::io::Error> {
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn move_cursor_to(p: Position) -> Result<(), std::io::Error> {
        queue!(stdout(), MoveTo(p.x, p.y))?;
        Ok(())
    }

    pub fn hide_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), Show)?;
        Ok(())
    }

    pub fn print(s: &str) -> Result<(), std::io::Error> {
        queue!(stdout(), Print(s))?;
        Ok(())
    }

    pub fn size() -> Result<Size, std::io::Error> {
        let (width, height) = size()?;
        Ok(Size { height, width })
    }

    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()?;
        Ok(())
    }
}
