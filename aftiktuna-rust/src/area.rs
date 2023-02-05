use crate::action::door::Door;
use crate::action::trade::Points;
use crate::action::CrewMember;
use crate::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::{Messages, NameData};
use crate::{action, item};
use door::DoorInfo;
use hecs::{Entity, World};
use rand::seq::index;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
pub use template::LocationData;

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
        let categories = load_location_categories()
            .unwrap_or_else(|message| panic!("Error loading \"locations.json\": {}", message));
        Locations {
            categories,
            count_until_win,
        }
    }

    pub fn single(location: String) -> Self {
        Locations {
            categories: vec![Category {
                name: "test".to_string(),
                location_names: vec![location],
            }],
            count_until_win: 1,
        }
    }

    pub fn next(&mut self, rng: &mut impl Rng) -> PickResult {
        if self.count_until_win <= 0 || self.categories.is_empty() {
            return PickResult::None;
        }

        match self.pick_category(rng) {
            Ok(category_index) => {
                PickResult::Location(self.pick_from_category(category_index, rng))
            }
            Err(choice) => PickResult::Choice(choice),
        }
    }

    fn pick_category(&self, rng: &mut impl Rng) -> Result<usize, Choice> {
        if self.categories.len() == 1 {
            return Ok(0);
        }

        let alternatives = index::sample(rng, self.categories.len(), 2)
            .into_iter()
            .map(|index| (index, self.categories[index].name.clone()))
            .collect::<Vec<_>>();

        println!("-----------");
        println!(
            "There are two destination targets: {}, {}",
            alternatives[0].1, alternatives[1].1
        );
        println!("Pick the location to travel to next.");
        println!();
        Err(Choice(alternatives))
    }

    pub fn try_make_choice(
        &mut self,
        choice: &Choice,
        input: String,
        rng: &mut impl Rng,
    ) -> Option<String> {
        match choice.try_choose(&input) {
            Some(category_index) => Some(self.pick_from_category(category_index, rng)),
            None => {
                println!("Unexpected input: \"{input}\"");
                None
            }
        }
    }

    fn pick_from_category(&mut self, category_index: usize, rng: &mut impl Rng) -> String {
        self.count_until_win -= 1;
        let category = self.categories.get_mut(category_index).unwrap();
        let chosen_location = category
            .location_names
            .remove(rng.gen_range(0..category.location_names.len()));
        if category.location_names.is_empty() {
            self.categories.remove(category_index);
        }
        chosen_location
    }
}

pub enum PickResult {
    None,
    Location(String),
    Choice(Choice),
}

pub struct Choice(Vec<(usize, String)>);

impl Choice {
    fn try_choose(&self, input: &str) -> Option<usize> {
        self.0
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(input))
            .map(|(index, _)| *index)
    }
}

pub fn load_location_categories() -> Result<Vec<Category>, String> {
    let file = File::open("assets/locations.json")
        .map_err(|error| format!("Failed to open file: {}", error))?;
    serde_json::from_reader::<_, Vec<Category>>(file)
        .map_err(|error| format!("Failed to parse file: {}", error))
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    name: String,
    pub location_names: Vec<String>,
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
    item::Type::Medkit.spawn(world, Pos::new(ship, 1, world));

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

    let start_pos = load_data(location_name)
        .and_then(|location_data| location_data.build(world))
        .unwrap_or_else(|message| panic!("Error loading location {}: {}", location_name, message));

    door::place_pair(
        world,
        DoorInfo(
            start_pos,
            'v',
            NameData::from_noun("ship entrance", "ship entrances"),
        ),
        DoorInfo(
            ship_exit,
            '^',
            NameData::from_noun("ship exit", "ship exits"),
        ),
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

    messages.add("The ship arrives at a new location, and the crew exit the ship.");
}

pub fn load_data(name: &str) -> Result<LocationData, String> {
    let file = File::open(format!("assets/{}.json", name))
        .map_err(|error| format!("Failed to open file: {}", error))?;
    serde_json::from_reader(file).map_err(|error| format!("Failed to parse file: {}", error))
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
