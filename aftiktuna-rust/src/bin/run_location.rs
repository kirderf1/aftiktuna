use aftiktuna::location::GenerationState;
use aftiktuna::{game_interface, standard_io_interface};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    let game = game_interface::setup_new_with(
        GenerationState::single(location).expect("Unable to initialize game"),
    );
    standard_io_interface::run(game);
}
