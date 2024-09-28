use std::io::{self, Read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() -> std::io::Result<()> {
    for b in io::stdin().bytes() {
        let c = b.unwrap() as char;
        print!("{}", c);
        if c == 'q' {
            disable_raw_mode().unwrap();
            break;
        }
    }
    
    Ok(())
}