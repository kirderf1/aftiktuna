use aftiktuna::location;
use hecs::World;

fn main() {
    match location::load_locations() {
        Ok(locations) => verify_locations(locations),
        Err(message) => println!("Failed to load \"locations.json\": {}", message),
    }
}

fn verify_locations(locations: location::Locations) {
    let mut failure_count = 0;
    for category in locations.categories {
        for location_name in category.location_names {
            let mut world = World::new();
            if let Err(message) = location::load_data(&location_name)
                .and_then(|location_data| location_data.build(&mut world))
            {
                println!("Failed to load location \"{}\": {}", location_name, message);
                failure_count += 1;
            }
        }
    }
    if failure_count == 0 {
        println!("All locations are OK!");
    }
}
