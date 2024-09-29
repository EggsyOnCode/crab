use crossterm::event::{read, Event, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers, KeyCode};
use std::io::Error;
mod terminal;
use terminal::{Terminal, Size, Position};

pub struct Editor {
    should_quit: bool,
    terminal : Terminal, // should be an interface
}

impl Editor {
    pub const fn default() -> Self {
        let terminal = Terminal::default();
        Self { should_quit: false , terminal: terminal}
    }
    pub fn run(&mut self) {
        self.terminal.initialize().unwrap();
        let result = self.repl();
        self.terminal.terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }
    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                _ => {
                    self.terminal.move_cursor(code, modifiers);
                },
            }
        }
    }
    fn refresh_screen(&self) -> Result<(), Error> {
        // self.terminal.hide_cursor()?;
        if self.should_quit {
            self.terminal.clear_screen()?;
            self.terminal.print("Goodbye.\r\n")?;
        } 
        self.terminal.show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }
}