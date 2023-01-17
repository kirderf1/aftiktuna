use aftiktuna::area;
use hecs::World;

fn main() {
    let mut failue_count = 0;
    for category in area::load_location_categories() {
        for location_name in category.location_names {
            let mut world = World::new();
            if let Err(message) = area::load_data(&location_name)
                .and_then(|location_data| location_data.build(&mut world))
            {
                println!("Failed to load \"{}\": {}", location_name, message);
                failue_count += 1;
            }
        }
    }
    if failue_count == 0 {
        println!("All locations are OK!");
    }
}
