use crate::action;
use crate::action::door::Door;
use crate::action::Aftik;
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

pub struct Locations {
    categories: Vec<Category>,
    count_until_win: i32,
}

impl Locations {
    pub fn new(count_until_win: i32) -> Self {
        Locations {
            categories: vec![
                Category {
                    location_names: vec!["location/goblin_forest", "location/eyesaur_forest"],
                },
                Category {
                    location_names: vec![
                        "location/abandoned_facility",
                        "location/abandoned_facility2",
                    ],
                },
            ],
            count_until_win,
        }
    }

    pub fn pick_random(&mut self, rng: &mut Rng) -> Option<&'static str> {
        if self.count_until_win <= 0 || self.categories.is_empty() {
            return None;
        }

        self.count_until_win -= 1;
        let category_index = rng.usize(..self.categories.len());
        let category = self.categories.get_mut(category_index).unwrap();
        let chosen_location = category
            .location_names
            .remove(rng.usize(..category.location_names.len()));
        if category.location_names.is_empty() {
            self.categories.remove(category_index);
        }
        Some(chosen_location)
    }
}

struct Category {
    location_names: Vec<&'static str>,
}

pub fn init(world: &mut World) -> (Entity, Pos) {
    let ship = world.spawn((
        Area {
            label: "Ship".to_string(),
            size: 4,
        },
        Ship(ShipStatus::NeedTwoCans),
    ));
    let ship_exit = Pos::new(ship, 3, world);

    creature::spawn_aftik(world, "Cerulean", Stats::new(9, 2, 10));
    let mint = creature::spawn_aftik(world, "Mint", Stats::new(10, 3, 8));

    (mint, ship_exit)
}

pub fn load_location(world: &mut World, ship_exit: Pos, location_name: &str) {
    let location = load_data(location_name);

    let start_pos = location
        .build(world)
        .unwrap_or_else(|message| panic!("{}", message));

    door::place_pair(
        world,
        DoorInfo(start_pos, DisplayInfo::from_noun('v', "ship entrance", 20)),
        DoorInfo(ship_exit, DisplayInfo::from_noun('^', "ship exit", 20)),
        None,
    );

    let aftiks = world
        .query::<()>()
        .with::<&Aftik>()
        .iter()
        .map(|pair| pair.0)
        .collect::<Vec<_>>();
    for aftik in aftiks {
        world.insert_one(aftik, start_pos).unwrap();
    }
}

fn load_data(name: &str) -> LocationData {
    let file = File::open(format!("assets/{}.json", name))
        .expect(&format!("Failed to load location: {}", name));
    serde_json::from_reader(file).unwrap()
}

struct Keep;

pub fn despawn_all_except_ship(world: &mut World, ship: Entity) {
    world.insert_one(ship, Keep).unwrap();
    let entities = world
        .query::<&Pos>()
        .without::<&Door>()
        .iter()
        .filter(|(_, pos)| pos.is_in(ship))
        .map(|pair| pair.0)
        .collect::<Vec<_>>();
    for entity in entities {
        world.insert_one(entity, Keep).unwrap();
        if let Some(item) = action::item::get_wielded(world, entity) {
            world.insert_one(item, Keep).unwrap();
        }
        for item in action::item::get_inventory(world, entity) {
            world.insert_one(item, Keep).unwrap();
        }
    }

    let entities = world
        .query::<()>()
        .without::<&Keep>()
        .iter()
        .map(|pair| pair.0)
        .collect::<Vec<_>>();
    for entity in entities {
        world.despawn(entity).unwrap();
    }

    let entities = world
        .query::<()>()
        .with::<&Keep>()
        .iter()
        .map(|pair| pair.0)
        .collect::<Vec<_>>();
    for entity in entities {
        world.remove_one::<Keep>(entity).unwrap();
    }
}
