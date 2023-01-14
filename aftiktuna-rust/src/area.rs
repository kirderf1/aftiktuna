use crate::action;
use crate::action::door::Door;
use crate::action::trade::Points;
use crate::action::CrewMember;
use crate::area::template::LocationData;
use crate::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::{DisplayInfo, Messages};
use door::DoorInfo;
use hecs::{Entity, World};
use rand::seq::index;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;

mod creature;
mod door;
mod init;
mod template;

pub struct Area {
    pub size: Coord,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct Ship {
    pub status: ShipStatus,
    pub exit_pos: Pos,
}

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
        let file = File::open("assets/locations.json").expect("Failed to open locations.json");
        let categories = serde_json::from_reader(file).expect("Failed to load locations.json");
        Locations {
            categories,
            count_until_win,
        }
    }

    pub fn pick_random(&mut self, rng: &mut impl Rng) -> Option<String> {
        if self.count_until_win <= 0 || self.categories.is_empty() {
            return None;
        }

        self.count_until_win -= 1;
        let category_index = self.pick_category(rng);
        let category = self.categories.get_mut(category_index).unwrap();
        let chosen_location = category
            .location_names
            .remove(rng.gen_range(0..category.location_names.len()));
        if category.location_names.is_empty() {
            self.categories.remove(category_index);
        }
        Some(chosen_location)
    }

    fn pick_category(&self, rng: &mut impl Rng) -> usize {
        if self.categories.len() == 1 {
            return 0;
        }

        let alternatives = index::sample(rng, self.categories.len(), 2)
            .into_iter()
            .map(|index| (index, &self.categories[index].name))
            .collect::<Vec<_>>();

        println!("-----------");
        println!(
            "There are two destination targets: {}, {}",
            alternatives[0].1, alternatives[1].1
        );
        println!("Pick the location to travel to next.");
        println!();

        loop {
            let input = crate::read_input().to_lowercase();

            for (index, name) in &alternatives {
                if input.eq(*name) {
                    return *index;
                }
            }
            println!("Unexpected input: \"{}\"", input);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Category {
    name: String,
    location_names: Vec<String>,
}

pub fn init(world: &mut World) -> (Entity, Entity) {
    let ship = world.spawn((Area {
        label: "Ship".to_string(),
        size: 4,
    },));
    world
        .insert_one(
            ship,
            Ship {
                status: ShipStatus::NeedTwoCans,
                exit_pos: Pos::new(ship, 3, world),
            },
        )
        .unwrap();

    let crew = world.spawn((Points(10000),));

    creature::spawn_crew_member(world, crew, "Cerulean", Stats::new(9, 2, 10));
    let mint = creature::spawn_crew_member(world, crew, "Mint", Stats::new(10, 3, 8));

    (mint, ship)
}

pub fn load_location(
    world: &mut World,
    messages: &mut Messages,
    ship: Entity,
    location_name: &str,
) {
    let ship_exit = world.get::<&Ship>(ship).unwrap().exit_pos;
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
        .with::<&CrewMember>()
        .iter()
        .map(|pair| pair.0)
        .collect::<Vec<_>>();
    for aftik in aftiks {
        world.insert_one(aftik, start_pos).unwrap();
    }

    messages.add("The ship arrives at a new location, and the crew exit the ship.".to_string());
}

fn load_data(name: &str) -> LocationData {
    let file = File::open(format!("assets/{}.json", name))
        .unwrap_or_else(|_| panic!("Failed to open location: {}", name));
    serde_json::from_reader(file).unwrap_or_else(|_| panic!("Failed to load location: {}", name))
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
        if let Ok(crew) = world
            .get::<&CrewMember>(entity)
            .map(|crew_member| crew_member.0)
        {
            world.insert_one(crew, Keep).unwrap();
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
