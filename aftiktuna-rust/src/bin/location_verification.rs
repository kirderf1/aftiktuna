use aftiktuna::asset::location::LocationData;
use aftiktuna::location;

fn main() {
    let locations = match location::Locations::load_from_json() {
        Ok(locations) => locations,
        Err(message) => {
            eprintln!("{message}");
            return;
        }
    };

    let mut failure_count = 0;
    for location_name in locations.all_location_names() {
        if !try_load(location_name) {
            failure_count += 1;
        }
    }

    if failure_count == 0 {
        println!("All locations are OK!");
    };
}

fn try_load(location_name: &str) -> bool {
    if let Err(message) = LocationData::load_from_json(location_name).and_then(|location_data| {
        location::generate::build_location(
            location_data,
            &mut location::LocationGenContext::default(),
        )
    }) {
        eprintln!("Failed to load location \"{location_name}\":");
        eprintln!("{message}");
        false
    } else {
        true
    }
}
