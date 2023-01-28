use aftiktuna::area::Locations;
use aftiktuna::game_loop;

fn main() {
    println!("Welcome to aftiktuna!");
    game_loop::run(Locations::new(3));
}
