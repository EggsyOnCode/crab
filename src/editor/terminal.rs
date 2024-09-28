use crossterm::event::{read, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers, Event};
use crossterm::queue;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, size};
use std::io::{stdout, Write};
use crossterm::cursor::MoveTo;
pub use crossterm::terminal::ClearType;

#[derive(Copy, Clone)]
pub struct ScreenSize {
    pub rows: u16,
    pub cols: u16,
}
pub struct Terminal {}

impl Terminal {
    pub fn terminate() -> Result<(), std::io::Error> {
        Self::execute()?;
        disable_raw_mode()
    }
    pub fn clear_screen(clear_type: ClearType) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        queue!(stdout, Clear(clear_type))
    }
    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen(ClearType::All)?;
        Self::move_cursor_to(0, 0)?;
        Self::execute()?;
        Ok(())
    }
    pub fn move_cursor_to(x: u16, y: u16) -> Result<(), std::io::Error> {
        queue!(stdout(), MoveTo(x, y))?;
        Ok(())
    }
    pub fn size() -> Result<ScreenSize, std::io::Error> {
        let (width,height) =  size()?;
        Ok((ScreenSize {rows: height, cols: width}))
    }
    pub fn hide_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), crossterm::cursor::Hide)
    }
    pub fn show_cursor() -> Result<(), std::io::Error> {
        queue!(stdout(), crossterm::cursor::Show)
    }
    pub fn print(s : &str) -> Result<(), std::io::Error> {
        queue!(stdout(), crossterm::style::Print(s))?;
        Ok(())
    }
    pub fn execute() -> Result<(), std::io::Error> {
        stdout().flush()?;
        Ok(())
    }
    
}