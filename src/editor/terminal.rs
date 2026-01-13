use super::{Position, Size};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Attribute, Print};
use crossterm::terminal::{
    Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
    SetTitle, disable_raw_mode, enable_raw_mode, size,
};
use crossterm::{Command, queue};
use std::io::{Write, stdout};

/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation
/// only spans over at most `usize::MAX` or `u16::size` rows/columns, whichever is smaller.
/// Each size returned truncates to min(`usize::MAX`, `u16::MAX`)
/// And should you attempt to set the caret out of these bounds, it will also be truncated.
pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(())
    }

    pub fn terminate() -> Result<(), std::io::Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    fn enter_alternate_screen() -> Result<(), std::io::Error> {
        Self::queue_command(EnterAlternateScreen)?;
        Ok(())
    }

    fn leave_alternate_screen() -> Result<(), std::io::Error> {
        Self::queue_command(LeaveAlternateScreen)?;
        Ok(())
    }

    fn disable_line_wrap() -> Result<(), std::io::Error> {
        Self::queue_command(DisableLineWrap)?;
        Ok(())
    }

    fn enable_line_wrap() -> Result<(), std::io::Error> {
        Self::queue_command(EnableLineWrap)?;
        Ok(())
    }

    fn clear_screen() -> Result<(), std::io::Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }

    /// Moves the caret to the given Position.
    /// # Arguments
    /// * `Position` - the `Position` to move the caret to. Will be truncated to `u16::MAX` if bigger.
    pub fn move_caret_to(p: &Position) -> Result<(), std::io::Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(p.col as u16, p.row as u16))?;
        Ok(())
    }

    pub fn hide_caret() -> Result<(), std::io::Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }

    pub fn show_caret() -> Result<(), std::io::Error> {
        Self::queue_command(Show)?;
        Ok(())
    }

    pub fn set_title(title: &str) -> Result<(), std::io::Error> {
        Self::queue_command(SetTitle(title))?;
        Ok(())
    }

    pub fn print(s: &str) -> Result<(), std::io::Error> {
        Self::queue_command(Print(s))?;
        Ok(())
    }

    pub fn print_row(row: usize, line_text: &str) -> Result<(), std::io::Error> {
        Self::move_caret_to(&Position { row, col: 0 })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        Ok(())
    }

    pub fn print_inverted_row(row: usize, line_text: &str) -> Result<(), std::io::Error> {
        let width = Self::size()?.width;
        Self::print_row(
            row,
            &format!(
                "{}{:width$.width$}{}",
                Attribute::Reverse,
                line_text,
                Attribute::Reset
            ),
        )
    }

    fn clear_line() -> Result<(), std::io::Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// Returns the current size of this Terminal.
    /// Edge Case for systems with `usize` < `u16`:
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated to `usize` if `usize` < `z` < `u16`
    pub fn size() -> Result<Size, std::io::Error> {
        let (width, height) = size()?;

        #[allow(clippy::as_conversions)]
        let height = height as usize;
        #[allow(clippy::as_conversions)]
        let width = width as usize;

        Ok(Size { height, width })
    }

    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), std::io::Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
