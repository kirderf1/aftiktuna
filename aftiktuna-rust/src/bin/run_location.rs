use aftiktuna::area::LocationTracker;
use aftiktuna::{game_loop, standard_io_interface};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    let game = game_loop::setup_new(LocationTracker::single(location));
    standard_io_interface::run(game, Default::default());
}
