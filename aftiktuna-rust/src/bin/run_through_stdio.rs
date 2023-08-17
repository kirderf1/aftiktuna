use aftiktuna::{game_loop, standard_io_interface};

fn main() {
    match game_loop::new_or_load() {
        Ok((game, frames)) => standard_io_interface::run(game, frames),
        Err(error) => eprintln!("Unable to load game: {error}"),
    }
}
