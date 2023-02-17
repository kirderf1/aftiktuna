use aftiktuna::area::Locations;
use aftiktuna::standard_io_interface;

fn main() {
    println!("Welcome to aftiktuna!");
    standard_io_interface::run(Locations::new(3));
}
