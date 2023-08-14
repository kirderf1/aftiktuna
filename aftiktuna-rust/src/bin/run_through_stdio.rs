use aftiktuna::area::LocationTracker;
use aftiktuna::standard_io_interface;

fn main() {
    standard_io_interface::run(LocationTracker::new(3));
}
