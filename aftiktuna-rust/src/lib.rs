use std::io;
use std::io::Write;

mod action;
mod ai;
pub mod area;
mod command;
pub mod game_loop;
mod item;
mod position;
mod status;
mod view;

pub fn read_input() -> String {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    String::from(input.trim())
}
