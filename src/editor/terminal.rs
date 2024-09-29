use crossterm::cursor::{Hide, MoveTo, Show, EnableBlinking, SetCursorStyle};
use crossterm::queue;
use crossterm::style::{Print, SetBackgroundColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::event::KeyCode;
use std::io::{stdout, Error, Write};
use crossterm::event::{read, Event, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers};
extern crate custom_error;
use custom_error::custom_error;
use std::fs::OpenOptions;

fn log_to_file(message: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("debug_log.txt")
        .unwrap();
    writeln!(file, "{}", message).unwrap();
    // panic!("pasting")
}


custom_error!{MyError
    TerminalInvalidPosition = "invalid position in copy_over_buffer",
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub height: u16,
    pub width: u16,
}

enum AppError {
    InvalidPosition,
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}
pub struct Terminal {
    viz_mode : bool,
    viz_mode_buffer : Vec<String>,
    viz_org_cursor_pos : Position,
    viz_cursor_pos : Position,
    pub t_size: Size,
    pub curr_pos : Position,      // (x,y) current pos in the buffer
    pub scroll_offest : Position, // (x,y) top left of the visible viewport
    pub buffer : Vec<String>,      // buffer to store the text
}

impl Terminal {
    pub const fn default() -> Self {
        Self {
            viz_mode: false,
            viz_mode_buffer: Vec::new(),
            viz_org_cursor_pos: Position{ x:0, y: 0},
            viz_cursor_pos: Position{ x:0, y: 0},
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
        queue!(stdout(), SetCursorStyle::BlinkingBlock)?;

        // Loop until 'Ctrl + q' is pressed
        while self.display_welcome_screen()? {}

        // Move cursor back to the top-left position
        self.move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?; // Execute any queued terminal commands
        queue!(stdout(), EnableBlinking)?;

        self.draw_rows(self.curr_pos)?; // Draw the initial rows of the editor
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

    fn copy_to_buffer(&mut self, from: Position, to: Position) -> Result<(), Error> {

        log_to_file(&format!("positions are: from {:?} to: {:?}", from, to));
        // Ensure valid positions
        if from.y >= self.buffer.len() as u16 || to.y >= self.buffer.len() as u16 {
            return Err(Error::new(std::io::ErrorKind::InvalidInput, "Invalid line indices"));
        }

        // Clear the viz_mode_buffer before copying
        self.viz_mode_buffer.clear();

        // Case 1: Copy within the same line
        if from.y == to.y {
            let line = &self.buffer[from.y as usize];
            if from.x <= to.x && to.x <= line.len() as u16 {
                let copied_slice = &line[from.x as usize..to.x as usize];
                self.viz_mode_buffer.push(copied_slice.to_string());
            } else {
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "Invalid 'x' positions"));
            }
        }
        // Case 2: Copy across multiple lines
        else {
            // Copy from 'from.x' to the end of 'from_line'
            let from_line = &self.buffer[from.y as usize];
            if from.x < from_line.len() as u16 {
                let copied_slice = &from_line[from.x as usize..];
                self.viz_mode_buffer.push(copied_slice.to_string());
            } else {
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "'from.x' is out of bounds"));
            }

            // Copy entire lines between 'from.y' and 'to.y'
            for y in (from.y + 1)..to.y {
                let line = &self.buffer[y as usize];
                self.viz_mode_buffer.push(line.clone());
            }

            // Copy from the beginning of 'to_line' to 'to.x'
            let to_line = &self.buffer[to.y as usize];
            if to.x <= to_line.len() as u16 {
                let copied_slice = &to_line[..to.x as usize];
                self.viz_mode_buffer.push(copied_slice.to_string());
            } else {
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "'to.x' is out of bounds"));
            }
        }


        Ok(())
    }



    fn handle_viz_mode(&mut self, code: &KeyCode, modifiers : &KeyModifiers) -> Result<(), Error> {
        match code {
                KeyCode::Up => {
                    if self.viz_cursor_pos.y > 0 {
                        self.viz_cursor_pos.y -= 1;
                        if self.viz_cursor_pos.y < self.scroll_offest.y {
                            self.scroll_offest.y = self.viz_cursor_pos.y;
                        }
                    }
                }
                KeyCode::Down => {
                    if self.viz_cursor_pos.y < self.buffer.len().saturating_sub(1) as u16 {
                        self.viz_cursor_pos.y += 1;
                        if self.viz_cursor_pos.y >= self.scroll_offest.y + self.t_size.height {
                            self.scroll_offest.y = self.viz_cursor_pos.y - self.t_size.height + 1;
                        }
                    } else {
                        // Optionally, add a new line if at the end
                        let line = self.buffer.get_mut(self.viz_cursor_pos.y as usize).unwrap();
                        let new_line = line.split_off(self.viz_cursor_pos.x as usize);
                        self.buffer.insert(self.viz_cursor_pos.y as usize + 1, new_line);
                        self.viz_cursor_pos.y += 1;
                        self.viz_cursor_pos.x = 0;

                    }
                }
                KeyCode::Left => {
                    if self.viz_cursor_pos.x > 0 {
                        self.viz_cursor_pos.x -= 1; // Move left
                    } else if self.viz_cursor_pos.y > 0 {
                        // If at the beginning of the line, move up to the last char of the previous line
                        self.viz_cursor_pos.y -= 1;
                        self.viz_cursor_pos.x = self.buffer[self.curr_pos.y as usize].len() as u16;
                    }
                }
                KeyCode::Right => {
                    if self.viz_cursor_pos.y < self.buffer.len() as u16 {
                        let line_len = self.buffer[self.viz_cursor_pos.y as usize].len() as u16;
                        if self.viz_cursor_pos.x < line_len {
                            self.viz_cursor_pos.x += 1; // Move right
                        } else if self.viz_cursor_pos.y < self.buffer.len().saturating_sub(1) as u16 {
                            // If at the end of the line, move down to the beginning of the next line
                            self.viz_cursor_pos.y += 1;
                            self.viz_cursor_pos.x = 0;
                        }
                    }
                }
                Char('c') if *modifiers == KeyModifiers::CONTROL => {
                    self.copy_to_buffer(self.viz_cursor_pos, self.viz_org_cursor_pos)?;
                    log_to_file(&format!("Copied buffer content: {:?}", self.viz_mode_buffer));
                }
                Char('b') if *modifiers == KeyModifiers::CONTROL => {
                    // Log buffer content to file before pasting
                    panic!("pasting");
                    log_to_file(&format!("Pasting buffer content: {:?}", self.viz_mode_buffer));

                    // Paste the buffer
                    let buffer = self.viz_mode_buffer.clone();
                    let mut y = self.viz_cursor_pos.y as usize;
                    for line_content in buffer {
                        if y < self.buffer.len() {
                            let line = self.buffer.get_mut(y).unwrap();
                            // Insert the copied content (line_content) at the cursor position
                            line.insert_str(self.viz_cursor_pos.x as usize, &line_content);
                        } else {
                            // If the line doesn't exist, append the content as a new line
                            self.buffer.push(line_content);
                        }
                        y += 1;
                    }
                    // Log buffer content to file after pasting
                    log_to_file(&format!("Buffer after pasting: {:?}", self.buffer));
                }


            KeyCode::Enter => {
                //leave the viz mode
                self.viz_mode = false;
                queue!(stdout(), SetCursorStyle::BlinkingBlock)?;
                Self::execute()?;
                self.viz_mode_buffer.clear();
                queue!(stdout(), crossterm::style::SetForegroundColor(crossterm::style::Color::White))?;
            }
            _ => ()
        }

        self.scroll_viewport()?;
        self.move_cursor_to(self.viz_cursor_pos)?;
        self.draw_rows(self.viz_cursor_pos)?;

        Ok(())
    }

    pub fn move_cursor(&mut self, code: &KeyCode, modifiers : &KeyModifiers) -> Result<(), Error> {
        if self.viz_mode {
            self.handle_viz_mode(code, modifiers);
        } else {
            match code {
                Char('v') if *modifiers == KeyModifiers::ALT => {
                    self.viz_mode = true;
                    self.viz_cursor_pos = self.curr_pos;
                    self.viz_org_cursor_pos = self.curr_pos;
                    queue!(stdout(), SetCursorStyle::BlinkingUnderScore)?;
                    queue!(stdout(), crossterm::style::SetForegroundColor(crossterm::style::Color::Red))?;
                    Self::execute()?
                }
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
                KeyCode::Backspace => {
                    // chars not at the start of the line
                    if self.curr_pos.x > 0 {
                        let line = self.buffer.get_mut(self.curr_pos.y as usize).unwrap();
                        line.remove(self.curr_pos.x as usize - 1);
                        self.curr_pos.x -= 1;
                    } else if self.curr_pos.y > 0 {
                        let line = self.buffer.remove(self.curr_pos.y as usize);
                        self.curr_pos.y -= 1;
                        self.curr_pos.x = self.buffer[self.curr_pos.y as usize].len() as u16;
                        self.buffer[self.curr_pos.y as usize].push_str(&line);
                    }
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
            self.draw_rows(self.curr_pos)?;
        }
        
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
        self.draw_rows(self.curr_pos)?; // Redraw the rows to reflect changes
        Ok(())
    }


    // Draw rows of the text editor
    fn draw_rows(&self, cur_pos : Position) -> Result<(), Error> {
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
                if buffer_y == cur_pos.y as usize {
                    self.move_cursor_to(cur_pos)?; // Move cursor to the correct position
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
        self.move_cursor_to(cur_pos)?;
        Self::execute()?;
        Ok(())
    }

    // Handle terminal resize events
    pub fn handle_resize(&mut self) -> Result<(), Error> {
        self.t_size = Self::size()?;
        self.scroll_viewport()?;
        self.draw_rows(self.curr_pos)?;
        self.move_cursor_to(self.curr_pos)?;
        Self::execute()?;
        Ok(())
    }

}
