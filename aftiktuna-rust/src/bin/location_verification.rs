use aftiktuna::area;
use hecs::World;

fn main() {
    for category in area::load_location_categories() {
        for location_name in category.location_names {
            let mut world = World::new();
            if let Err(message) = area::load_data(&location_name)
                .and_then(|location_data| location_data.build(&mut world))
            {
                println!("{}", message);
            }
        }
    }
}
