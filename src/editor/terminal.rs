use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use std::io::{stdout, Error, Write};
use crossterm::event::{read, Event, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers};

#[derive(Copy, Clone)]
pub struct Size {
    pub height: u16,
    pub width: u16,
}
#[derive(Copy, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}
pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }
    pub fn display_welcome_screen() -> Result<bool, Error> {
        let (width, height) = size()?; // Get the terminal size
        Self::move_cursor_to(Position { x: width / 2, y: height / 2 })?; // Move cursor to center
        Self::print("Welcome to Crab, your fav text editor!!")?; // Display the welcome message
        Self::execute()?; // Execute the print command

        // Wait for a key event
        let event = read()?; // Read the event from stdin

        // Match the event
        match event {
            Key(KeyEvent { code, modifiers, .. }) => {
                // Check if the key is 'q' with Control modifier
                if code == Char('q') && modifiers == KeyModifiers::CONTROL {
                    return Ok(true); // Return true if 'Ctrl + q' is pressed
                }
            }
            _ => {}
        }

        Ok(false) // Return false if no valid key is pressed
    }

    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        while (Self::display_welcome_screen()?) {}
        Self::move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?;
        Ok(())
    }
    pub fn clear_screen() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }
    pub fn clear_line() -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }
    pub fn move_cursor_to(position: Position) -> Result<(), Error> {
        queue!(stdout(), MoveTo(position.x, position.y))?;
        Ok(())
    }
    pub fn hide_cursor() -> Result<(), Error> {
        queue!(stdout(), Hide)?;
        Ok(())
    }
    pub fn show_cursor() -> Result<(), Error> {
        queue!(stdout(), Show)?;
        Ok(())
    }
    pub fn print(string: &str) -> Result<(), Error> {
        queue!(stdout(), Print(string))?;
        Ok(())
    }
    pub fn size() -> Result<Size, Error> {
        let (width, height) = size()?;
        Ok(Size { height, width })
    }
    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
}