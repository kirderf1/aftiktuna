use aftiktuna::{game_interface, standard_io_interface};

fn main() {
    match game_interface::new_or_load() {
        Ok(game) => standard_io_interface::run(game),
        Err(error) => eprintln!("Unable to load game: {error}"),
    }
}
