use aftiktuna::{game_loop, standard_io_interface};

fn main() {
    let game = game_loop::new_or_load();
    standard_io_interface::run(game);
}
