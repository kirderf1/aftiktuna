use crate::core::area::{Area, BackgroundType, FuelAmount, Ship, ShipControls, ShipStatus};
use crate::core::position::{Direction, Pos};
use crate::core::status::Stats;
use crate::core::{inventory, item, Aggressive, CrewMember, Door, DoorKind, Points, Threatening};
use crate::game_loop::GameState;
use crate::view::area::{AftikColorId, OrderWeight, Symbol, TextureType};
use crate::view::name::Noun;
use crate::view::Messages;
use door::DoorInfo;
use hecs::{CommandBuffer, Entity, World};
use rand::seq::index;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
pub use template::LocationData;

mod creature;
mod door;
mod template;

#[derive(Eq, PartialEq, Serialize, Deserialize)]
enum TrackedState {
    BeforeFortuna { remaining_locations_count: i32 },
    AtFortuna,
}

#[derive(Serialize, Deserialize)]
pub struct LocationTracker {
    locations: Locations,
    state: TrackedState,
}

impl LocationTracker {
    pub fn new(locations_before_fortuna: i32) -> Self {
        Self {
            locations: load_locations()
                .unwrap_or_else(|message| panic!("Error loading \"locations.json\": {}", message)),
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: locations_before_fortuna,
            },
        }
    }

    pub fn single(location: String) -> Self {
        Self {
            locations: Locations::single(location),
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: 1,
            },
        }
    }

    pub fn next(&mut self, rng: &mut impl Rng) -> PickResult {
        match &mut self.state {
            TrackedState::AtFortuna => PickResult::None,
            TrackedState::BeforeFortuna {
                remaining_locations_count,
            } => {
                if *remaining_locations_count <= 0 {
                    self.state = TrackedState::AtFortuna;
                    return self.locations.pick_fortuna_location(rng);
                }

                *remaining_locations_count -= 1;
                self.locations.next(rng)
            }
        }
    }

    pub fn try_make_choice(
        &mut self,
        choice: &Choice,
        input: &str,
        rng: &mut impl Rng,
    ) -> Result<String, String> {
        let category_index = choice
            .try_choose(input)
            .ok_or_else(|| format!("Unexpected input: \"{input}\""))?;

        Ok(self.locations.pick_from_category(category_index, rng))
    }

    pub fn is_at_fortuna(&self) -> bool {
        self.state == TrackedState::AtFortuna
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Locations {
    pub categories: Vec<Category>,
    pub fortuna_locations: Vec<String>,
}

impl Locations {
    fn single(location: String) -> Self {
        Locations {
            categories: vec![Category {
                name: "test".to_string(),
                location_names: vec![location],
            }],
            fortuna_locations: vec![],
        }
    }

    fn next(&mut self, rng: &mut impl Rng) -> PickResult {
        if self.categories.is_empty() {
            return PickResult::None;
        }

        if self.categories.len() == 1 {
            return PickResult::Location(self.pick_from_category(0, rng));
        }

        let alternatives = index::sample(rng, self.categories.len(), 2)
            .into_iter()
            .map(|index| (index, self.categories[index].name.clone()))
            .collect::<Vec<_>>();

        PickResult::Choice(Choice(alternatives))
    }

    fn pick_from_category(&mut self, category_index: usize, rng: &mut impl Rng) -> String {
        let category = self.categories.get_mut(category_index).unwrap();
        let chosen_location = category
            .location_names
            .remove(rng.gen_range(0..category.location_names.len()));
        if category.location_names.is_empty() {
            self.categories.remove(category_index);
        }
        chosen_location
    }

    fn pick_fortuna_location(&mut self, rng: &mut impl Rng) -> PickResult {
        if self.fortuna_locations.is_empty() {
            return PickResult::None;
        }
        let location = self
            .fortuna_locations
            .remove(rng.gen_range(0..self.fortuna_locations.len()));
        PickResult::Location(location)
    }
}

pub enum PickResult {
    None,
    Location(String),
    Choice(Choice),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice(Vec<(usize, String)>);

impl Choice {
    pub fn present(&self) -> Messages {
        let alternatives = &self.0;
        let mut messages = Messages::default();
        messages.add(format!(
            "On the next planet, there are two destination targets: {}, {}",
            alternatives[0].1, alternatives[1].1
        ));
        messages.add("Pick the location to travel to next.");
        messages
    }

    pub fn alternatives(&self) -> Vec<String> {
        self.0.iter().map(|(_, name)| name.clone()).collect()
    }
}

impl Choice {
    fn try_choose(&self, input: &str) -> Option<usize> {
        self.0
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(input))
            .map(|(index, _)| *index)
    }
}

pub fn load_locations() -> Result<Locations, String> {
    let file = File::open("assets/locations.json")
        .map_err(|error| format!("Failed to open file: {}", error))?;
    serde_json::from_reader(file).map_err(|error| format!("Failed to parse file: {}", error))
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    name: String,
    pub location_names: Vec<String>,
}

pub fn init(world: &mut World) -> (Entity, Entity) {
    let ship = world.spawn((Area {
        label: "Ship".to_string(),
        size: 5,
        background: BackgroundType::new("ship"),
        background_offset: None,
    },));
    world
        .insert_one(
            ship,
            Ship {
                status: ShipStatus::NeedFuel(FuelAmount::TwoCans),
                exit_pos: Pos::new(ship, 3, world),
                item_pos: Pos::new(ship, 4, world),
            },
        )
        .unwrap();
    item::Type::Medkit.spawn(world, Pos::new(ship, 1, world));
    item::Type::FoodRation.spawn(world, Pos::new(ship, 4, world));
    world.spawn((
        Symbol('#'),
        TextureType::new("ship_controls"),
        OrderWeight::Background,
        Noun::new("ship controls", "ship controls"),
        Pos::new(ship, 0, world),
        ShipControls,
    ));

    let crew = world.spawn((Points(10000),));

    creature::spawn_crew_member(
        world,
        crew,
        "Cerulean",
        Stats::new(9, 2, 10),
        AftikColorId::new("cerulean"),
    );
    let mint = creature::spawn_crew_member(
        world,
        crew,
        "Mint",
        Stats::new(10, 3, 8),
        AftikColorId::new("mint"),
    );
    world.insert_one(mint, OrderWeight::Controlled).unwrap();

    (mint, ship)
}

pub fn load_location(state: &mut GameState, messages: &mut Messages, location_name: &str) {
    let world = &mut state.world;
    let ship_exit = world.get::<&Ship>(state.ship).unwrap().exit_pos;

    let start_pos = load_data(location_name)
        .and_then(|location_data| location_data.build(world, &mut state.rng))
        .unwrap_or_else(|message| panic!("Error loading location {}: {}", location_name, message));

    door::place_pair(
        world,
        DoorInfo {
            pos: start_pos,
            symbol: Symbol('v'),
            texture_type: TextureType::new("ship"),
            kind: DoorKind::Door,
            name: Noun::new("ship", "ships"),
        },
        DoorInfo {
            pos: ship_exit,
            symbol: Symbol('^'),
            texture_type: TextureType::new("ship_exit"),
            kind: DoorKind::Door,
            name: Noun::new("ship exit", "ship exits"),
        },
        None,
    );

    let aftiks = world
        .query::<()>()
        .with::<&CrewMember>()
        .iter()
        .map(|pair| pair.0)
        .collect::<Vec<_>>();
    let direction = Direction::towards_center(start_pos, world);
    for aftik in aftiks {
        world.insert(aftik, (start_pos, direction)).unwrap();
    }

    let mut buffer = CommandBuffer::new();
    for (entity, pos) in world.query::<&Pos>().with::<&Threatening>().iter() {
        if world
            .query::<&Pos>()
            .with::<&Aggressive>()
            .iter()
            .any(|(_, other_pos)| other_pos.is_in(pos.get_area()))
        {
            buffer.remove_one::<Threatening>(entity);
            buffer.insert_one(entity, Aggressive);
        }
    }
    buffer.run_on(world);

    if state.locations.is_at_fortuna() {
        messages.add(
            "The ship arrives at the location of the fortuna chest, and the crew exit the ship.",
        )
    } else {
        messages.add("The ship arrives at a new location, and the crew exit the ship.");
    }
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
        if let Some(item) = inventory::get_wielded(world, entity) {
            world.insert_one(item, Keep).unwrap();
        }
        for item in inventory::get_inventory(world, entity) {
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
