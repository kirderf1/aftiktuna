use aftiktuna::location;
use hecs::World;

fn main() {
    match location::load_locations() {
        Ok(locations) => verify_locations(locations),
        Err(message) => println!("Failed to load \"locations.json\": {message}"),
    }
}

fn verify_locations(locations: location::Locations) {
    let mut failure_count = 0;
    for category in &locations.categories {
        for location_name in &category.location_names {
            if !try_load(location_name) {
                failure_count += 1;
            }
        }
    }
    for location_name in &locations.fortuna_locations {
        if !try_load(location_name) {
            failure_count += 1;
        }
    }

    if failure_count == 0 {
        println!("All locations are OK!");
    }
}

fn try_load(location_name: &str) -> bool {
    let mut world = World::new();
    let mut rng = rand::thread_rng();
    if let Err(message) = location::load_data(location_name)
        .and_then(|location_data| location_data.build(&mut world, &mut rng))
    {
        println!("Failed to load location \"{location_name}\": {message}");
        false
    } else {
        true
    }
}
