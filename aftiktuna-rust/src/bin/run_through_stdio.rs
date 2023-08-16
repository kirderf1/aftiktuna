use aftiktuna::area::LocationTracker;
use aftiktuna::{game_loop, standard_io_interface};

fn main() {
    let game = game_loop::setup(LocationTracker::new(3));
    standard_io_interface::run(game);
}
