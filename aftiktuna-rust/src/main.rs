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
