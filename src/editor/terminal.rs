use crossterm::cursor::{Hide, MoveTo, Show, EnableBlinking};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::event::KeyCode;
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
pub struct Terminal {
    pub t_size: Size,
    pub curr_pos : Position,      // (x,y) current pos in the buffer
    pub scroll_offest : Position, // (x,y) top left of the visible viewport
    pub buffer : Vec<String>,      // buffer to store the text
}

impl Terminal {
    pub const fn default() -> Self {
        Self {
            t_size: Size { height: 0, width: 0 } ,
            curr_pos : Position { x: 0, y: 0 },
            scroll_offest : Position { x: 0, y: 0 },
            buffer : Vec::new(),
        }
    }
    // Terminate the terminal, resetting modes
    pub fn terminate(&self) -> Result<(), Error> {
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }

    // Display the welcome screen and check for 'Ctrl + q' to exit
    pub fn display_welcome_screen(&self) -> Result<bool, Error> {
        let (width, height) = size()?; // Get the terminal size
        self.move_cursor_to(Position { x: width / 2, y: height / 2 })?; // Move cursor to center
        self.print("Welcome to Crab, your fav text editor!!")?; // Display welcome message
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

    // Initialize the terminal, enter raw mode, display the welcome screen, and record terminal size
    pub fn initialize(&mut self) -> Result<(), Error> {
        enable_raw_mode()?; // Enable raw mode
        self.clear_screen()?; // Clear the screen

        // Loop until 'Ctrl + q' is pressed
        while self.display_welcome_screen()? {}

        // Move cursor back to the top-left position
        self.move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?; // Execute any queued terminal commands
        queue!(stdout(), EnableBlinking)?;

        self.draw_rows()?; // Draw the initial rows of the editor
        self.move_cursor_to(Position { x: 0, y: 0 })?; // Move cursor to top-left

        // Update and record the terminal size
        self.t_size = Self::size()?;

        Ok(())
    }

    // Clears the entire terminal screen
    pub fn clear_screen(&self) -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    // Clears the current line in the terminal
    pub fn clear_line(&self) -> Result<(), Error> {
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    // Move the cursor to a specific position on the screen
    pub fn move_cursor_to(&self, position: Position) -> Result<(), Error> {
        queue!(stdout(), MoveTo(position.x, position.y))?;
        Ok(())
    }

    // Hide the terminal cursor
    pub fn hide_cursor(&self) -> Result<(), Error> {
        queue!(stdout(), Hide)?;
        Ok(())
    }

    // Show the terminal cursor
    pub fn show_cursor(&self) -> Result<(), Error> {
        queue!(stdout(), Show)?;
        Ok(())
    }

    // Print a string to the terminal
    pub fn print(&self, string: &str) -> Result<(), Error> {
        queue!(stdout(), Print(string))?;
        Ok(())
    }

    // Get the terminal's size (width and height)
    pub fn size() -> Result<Size, Error> {
        let (width, height) = size()?;
        Ok(Size { height, width })
    }

    // Flush the stdout buffer to execute any queued terminal commands
    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    pub fn move_cursor(&mut self, code: &KeyCode) -> Result<(), Error> {
        match code {
            KeyCode::Up => {
                if self.curr_pos.y > 0 {
                    self.curr_pos.y -= 1;
                    if self.curr_pos.y < self.scroll_offest.y {
                        self.scroll_offest.y = self.curr_pos.y;
                    }
                }
            }
            KeyCode::Down => {
                if self.curr_pos.y < self.buffer.len().saturating_sub(1) as u16 {
                    self.curr_pos.y += 1;
                    if self.curr_pos.y >= self.scroll_offest.y + self.t_size.height {
                        self.scroll_offest.y = self.curr_pos.y - self.t_size.height + 1;
                    }
                } else {
                    // Optionally, add a new line if at the end
                    let line = self.buffer.get_mut(self.curr_pos.y as usize).unwrap();
                    let new_line = line.split_off(self.curr_pos.x as usize);
                    self.buffer.insert(self.curr_pos.y as usize + 1, new_line);
                    self.curr_pos.y += 1;
                    self.curr_pos.x = 0;

                }
            }
            KeyCode::Left => {
                if self.curr_pos.x > 0 {
                    self.curr_pos.x -= 1; // Move left
                } else if self.curr_pos.y > 0 {
                    // If at the beginning of the line, move up to the last char of the previous line
                    self.curr_pos.y -= 1;
                    self.curr_pos.x = self.buffer[self.curr_pos.y as usize].len() as u16;
                }
            }
            KeyCode::Right => {
                if self.curr_pos.y < self.buffer.len() as u16 {
                    let line_len = self.buffer[self.curr_pos.y as usize].len() as u16;
                    if self.curr_pos.x < line_len {
                        self.curr_pos.x += 1; // Move right
                    } else if self.curr_pos.y < self.buffer.len().saturating_sub(1) as u16 {
                        // If at the end of the line, move down to the beginning of the next line
                        self.curr_pos.y += 1;
                        self.curr_pos.x = 0;
                    }
                }
            }
            KeyCode::Enter => {
                let line = self.buffer.get_mut(self.curr_pos.y as usize).unwrap();
                let new_line = line.split_off(self.curr_pos.x as usize);
                self.buffer.insert(self.curr_pos.y as usize + 1, new_line);
                self.curr_pos.y += 1;
                self.curr_pos.x = 0;
            }
            _ => {
                if let KeyCode::Char(c) = code {
                    self.insert_char(*c);
                }
            }
        }
        
        // Scroll the viewport and redraw
        self.scroll_viewport()?;
        self.move_cursor_to(self.curr_pos)?;
        self.draw_rows()?;
        Ok(())
    }


    fn scroll_viewport(&mut self) -> Result<(), Error> {
        // Ensure scroll_offset.y is within buffer bounds
        if self.scroll_offest.y > self.buffer.len().saturating_sub(1) as u16 {
            self.scroll_offest.y = self.buffer.len().saturating_sub(1) as u16;
        }

        // Ensure the viewport doesn't exceed terminal size
        if self.scroll_offest.y + self.t_size.height > self.buffer.len() as u16 {
            if self.buffer.len() as u16 >= self.t_size.height {
                self.scroll_offest.y = self.buffer.len() as u16 - self.t_size.height;
            } else {
                self.scroll_offest.y = 0;
            }
        }


        Ok(())
    }

    pub fn insert_char(&mut self, c: char) -> Result<(), Error> {
        // Ensure the current line exists, or create it if it doesn't
        if self.curr_pos.y as usize >= self.buffer.len() {
            self.buffer.push(String::new());
        }
        
        let line = self.buffer.get_mut(self.curr_pos.y as usize).unwrap();
        
        // Insert character at the current position
        line.insert(self.curr_pos.x as usize, c);
        self.curr_pos.x += 1;

        // Ensure the cursor doesn't go beyond the end of the line
        if self.curr_pos.x > line.len() as u16 {
            self.curr_pos.x = line.len() as u16; // Set cursor to the end of the line
        }

        self.move_cursor_to(self.curr_pos)?;
        Self::execute()?;
        self.draw_rows()?; // Redraw the rows to reflect changes
        Ok(())
    }


    // Draw rows of the text editor
    fn draw_rows(&self) -> Result<(), Error> {
        let start = self.scroll_offest.y as usize;
        let end = (self.scroll_offest.y + self.t_size.height).min(self.buffer.len() as u16) as usize;

        for y in start..end {
            let buffer_y = y;

            self.move_cursor_to(Position { x: 0, y: (y - start) as u16 })?; // Adjust cursor position
            self.clear_line()?;

            if buffer_y < self.buffer.len() {
                let line = &self.buffer[buffer_y];
                let display_line = if line.len() as u16 > self.t_size.width {
                    &line[..self.t_size.width as usize]
                } else {
                    line
                };
                self.print(display_line)?;

                // Highlight the cursor position if it’s on this line
                if buffer_y == self.curr_pos.y as usize {
                    self.move_cursor_to(self.curr_pos)?; // Move cursor to the correct position
                    self.print("^")?; // Use a character to indicate cursor position
                }
            } else {
                self.print("~")?; // Indicate empty lines
            }

            if y + 1 < end {
                self.print("\r\n")?;
            }
        }

        // After drawing rows, move the cursor to the actual position
        self.move_cursor_to(self.curr_pos)?;
        Self::execute()?;
        Ok(())
    }

    // Handle terminal resize events
    pub fn handle_resize(&mut self) -> Result<(), Error> {
        self.t_size = Self::size()?;
        self.scroll_viewport()?;
        self.draw_rows()?;
        self.move_cursor_to(self.curr_pos)?;
        Self::execute()?;
        Ok(())
    }

}
