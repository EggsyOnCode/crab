use crossterm::event::{read, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub struct Editor {
    should_quit : bool,
}

impl Editor {
    pub fn default() -> Self {
        Editor { should_quit: false }
    }
    pub fn run(&mut self) {
        if let Err(err) = self.repl() {
            panic!("Error: {}", err);
        }
        print!("Goodbye!\r\n");
    }

    fn repl(&mut self) -> Result<(), std::io::Error> {
        println!("Running editor");
        enable_raw_mode().unwrap();
        loop {
            // read() reads a single byte from the stdin tty
            if let Key(KeyEvent {
                code, modifiers, kind, state
            }) = read()?
            {
                println!("Code: {code:?} Modifiers: {modifiers:?} Kind: {kind:?} State: {state:?} \r");
                match code {
                    // is char q && modifiers inlcude CTRL flag
                    Char('q') if modifiers == KeyModifiers::CONTROL => {
                        self.should_quit = true;
                    }
                    _ => (),
                }
            }
            if self.should_quit {
                break;
            }
        }
        disable_raw_mode().unwrap();
        Ok(())
    }

}