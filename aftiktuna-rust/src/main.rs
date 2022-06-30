mod action;
mod area;
mod game_loop;
mod command;
mod position;
mod status;
mod view;

fn main() {
    println!("Welcome to aftiktuna!");
    game_loop::run();
}
