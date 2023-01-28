use aftiktuna::area::Locations;
use aftiktuna::game_loop;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    game_loop::run(Locations::single(location))
}
