use crate::area::template::LocationData;
use crate::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::DisplayInfo;
use door::DoorInfo;
use fastrand::Rng;
use hecs::{Entity, World};
use std::fs::File;

mod creature;
mod door;
mod init;
mod item;
mod template;

pub fn pick_random(rng: &mut Rng) -> &'static str {
    let locations = vec![
        "location/goblin_forest",
        "location/eyesaur_forest",
        "location/abandoned_facility",
        "location/abandoned_facility2",
    ];
    locations[rng.usize(..locations.len())]
}

pub fn init(world: &mut World, location_name: &str) -> Entity {
    let location = load_location(location_name);

    let start_pos = location
        .build(world)
        .unwrap_or_else(|message| panic!("{}", message));

    let ship = world.spawn((
        Area {
            label: "Ship".to_string(),
            size: 4,
        },
        Ship(ShipStatus::NeedTwoCans),
    ));
    door::place_pair(
        world,
        DoorInfo(start_pos, DisplayInfo::from_noun('v', "ship entrance", 20)),
        DoorInfo(
            Pos::new(ship, 3, world),
            DisplayInfo::from_noun('^', "ship exit", 20),
        ),
        None,
    );

    creature::place_aftik(world, start_pos, "Cerulean", Stats::new(9, 2, 10));
    creature::place_aftik(world, start_pos, "Mint", Stats::new(10, 3, 8))
}

fn load_location(name: &str) -> LocationData {
    let file = File::open(format!("assets/{}.json", name))
        .expect(&format!("Failed to load location: {}", name));
    serde_json::from_reader(file).unwrap()
}

pub struct Area {
    pub size: Coord,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct Ship(pub ShipStatus);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShipStatus {
    NeedTwoCans,
    NeedOneCan,
    Launching,
}
