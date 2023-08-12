use aftiktuna::area::LocationTracker;
use aftiktuna::standard_io_interface;

fn main() {
    println!("Welcome to aftiktuna!");
    standard_io_interface::run(LocationTracker::new(3));
}
