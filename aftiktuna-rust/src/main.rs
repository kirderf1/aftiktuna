use std::io;
use std::io::Write;

mod action;
mod ai;
mod area;
mod command;
mod game_loop;
mod position;
mod status;
mod view;

fn main() {
    println!("Welcome to aftiktuna!");
    game_loop::run();
}

fn read_input() -> String {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    String::from(input.trim())
}
