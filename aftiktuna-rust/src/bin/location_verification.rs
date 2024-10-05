use aftiktuna::location;
use hecs::World;

fn main() {
    match location::Locations::load_from_json() {
        Ok(locations) => verify_locations(locations),
        Err(message) => {
            eprintln!("Failed to load \"locations.json\":");
            eprintln!("{message}");
        }
    }
}

fn verify_locations(locations: location::Locations) {
    let mut failure_count = 0;
    for location_name in locations.all_location_names() {
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
    if let Err(message) = location::LocationData::load_from_json(location_name)
        .and_then(|location_data| location_data.build(&mut world, &mut Vec::default(), &mut rng))
    {
        eprintln!("Failed to load location \"{location_name}\":");
        eprintln!("{message}");
        false
    } else {
        true
    }
}
