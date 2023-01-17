use aftiktuna::area;
use aftiktuna::area::Category;
use hecs::World;

fn main() {
    match area::load_location_categories() {
        Ok(categories) => verify_locations(categories),
        Err(message) => println!("Failed to load \"locations.json\": {}", message),
    }
}

fn verify_locations(categories: Vec<Category>) {
    let mut failure_count = 0;
    for category in categories {
        for location_name in category.location_names {
            let mut world = World::new();
            if let Err(message) = area::load_data(&location_name)
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
