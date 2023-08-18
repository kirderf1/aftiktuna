use aftiktuna::{game_loop, standard_io_interface};

fn main() {
    match game_loop::new_or_load() {
        Ok((game, frame_cache)) => standard_io_interface::run(game, frame_cache),
        Err(error) => eprintln!("Unable to load game: {error}"),
    }
}
