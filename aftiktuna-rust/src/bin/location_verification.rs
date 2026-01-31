use aftiktuna::asset::GameAssets;
use aftiktuna::asset::location::LocationData;
use aftiktuna::location;
use aftiktuna::location::generate::LocationBuildData;

fn main() {
    let locations = match location::Locations::load_from_json() {
        Ok(locations) => locations,
        Err(message) => {
            eprintln!("{message}");
            return;
        }
    };
    let assets = GameAssets::load().unwrap();

    let mut failure_count = 0;
    for location_name in locations.all_location_names() {
        if !try_load(location_name, |_| Ok(()), &assets) {
            failure_count += 1;
        }
    }

    if !try_load(
        "crew_ship",
        |build_data| {
            if build_data.food_deposit_pos.is_none() {
                Err("Missing food deposit in ship".to_owned())
            } else if build_data.ship_dialogue_spot.is_none() {
                Err("Missing ship dialogue pos in ship".to_owned())
            } else {
                Ok(())
            }
        },
        &assets,
    ) {
        failure_count += 1;
    }

    if failure_count == 0 {
        println!("All locations are OK!");
    };
}

fn try_load(
    location_name: &str,
    verify_build_data: impl Fn(LocationBuildData) -> Result<(), String>,
    assets: &GameAssets,
) -> bool {
    let load_result = LocationData::load_from_json(location_name)
        .and_then(|location_data| {
            location::generate::build_location(
                location_data,
                &mut location::LocationGenContext::dummy(assets),
            )
        })
        .and_then(verify_build_data);

    if let Err(message) = load_result {
        eprintln!("Failed to load location \"{location_name}\":");
        eprintln!("{message}");
        false
    } else {
        true
    }
}
