use aftiktuna::area::LocationTracker;
use aftiktuna::standard_io_interface;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let location = args[1].to_string();
    standard_io_interface::run(LocationTracker::single(location));
}
