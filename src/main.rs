#![warn(clippy::all, clippy::pedantic)]
mod editor;
use editor::Editor;
use std::env;

fn main()  {
    env::set_var("RUST_BACKTRACE", "1");
    let mut editor = Editor::default();
    editor.run();
}