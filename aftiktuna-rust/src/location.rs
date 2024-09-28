use crate::core::area::{Area, BackgroundId, FuelAmount, Ship, ShipControls, ShipStatus};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::{self, Direction, Pos};
use crate::core::store::Points;
use crate::core::{inventory, item, CrewMember, Door, DoorKind, Hostile, Waiting};
use crate::game_loop::GameState;
use crate::serialization;
use crate::view::text::Messages;
use creature::{AftikProfile, ProfileOrRandom};
use door::DoorInfo;
use hecs::{CommandBuffer, Entity, Satisfies, World};
use rand::rngs::ThreadRng;
use rand::seq::index;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
pub struct GenerationState {
    locations: Locations,
    state: TrackedState,
    #[serde(default = "load_profiles_or_default")]
    character_profiles: Vec<AftikProfile>,
}

impl GenerationState {
    pub fn new(locations_before_fortuna: i32) -> Self {
        Self {
            locations: load_locations()
                .unwrap_or_else(|message| panic!("Error loading \"locations.json\": {message}")),
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: locations_before_fortuna,
            },
            character_profiles: load_profiles_or_default(),
        }
    }

    pub fn single(location: String) -> Self {
        Self {
            locations: Locations::single(location),
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: 1,
            },
            character_profiles: load_profiles_or_default(),
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
            .swap_remove(rng.gen_range(0..category.location_names.len()));
        if category.location_names.is_empty() {
            self.categories.swap_remove(category_index);
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
    pub fn presentation_text_lines(&self) -> Vec<String> {
        let Choice(alternatives) = self;
        let mut text_lines = Vec::new();
        text_lines.push(format!(
            "On the next planet, there are two destination targets: {}, {}",
            alternatives[0].1, alternatives[1].1
        ));
        text_lines.push("Pick the location to travel to next.".into());
        text_lines
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
    crate::load_json_simple("locations.json")
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    name: String,
    pub location_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CrewData {
    points: i32,
    crew: Vec<ProfileOrRandom>,
}

impl CrewData {
    pub fn load_starting_crew() -> Result<CrewData, String> {
        crate::load_json_simple("starting_crew.json")
    }
}

fn load_profiles_or_default() -> Vec<AftikProfile> {
    load_character_profiles().unwrap_or_else(|message| {
        eprintln!("Problem loading \"character_profiles.json\": {message}");
        Vec::default()
    })
}

fn load_character_profiles() -> Result<Vec<AftikProfile>, String> {
    crate::load_json_simple("character_profiles.json")
}

pub fn init(
    world: &mut World,
    crew_data: CrewData,
    generation_state: &mut GenerationState,
    rng: &mut impl Rng,
) -> (Entity, Entity) {
    let ship = world.spawn((Area {
        label: "Ship".to_string(),
        size: 5,
        background: BackgroundId::new("ship"),
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
        ModelId::new("ship_controls"),
        OrderWeight::Background,
        Noun::new("ship controls", "ship controls"),
        Pos::new(ship, 0, world),
        ShipControls,
    ));

    let crew = world.spawn((Points(crew_data.points),));

    let mut crew_iter = crew_data
        .crew
        .into_iter()
        .filter_map(|profile| profile.unwrap(&mut generation_state.character_profiles, rng));

    let mut iter_pos = Pos::new(ship, 0, world);
    let controlled = world.spawn(
        creature::aftik_builder_with_stats(crew_iter.next().expect("Crew must not be empty"), true)
            .add_bundle((CrewMember(crew), OrderWeight::Controlled, iter_pos))
            .build(),
    );
    'add_crew: for profile in crew_iter {
        while position::check_is_pos_blocked(iter_pos, world).is_err() {
            let Some(new_pos) = iter_pos.try_offset(1, world) else {
                eprintln!("Tried initializing a crew that is too large. Not all crew members will be added.");
                break 'add_crew;
            };
            iter_pos = new_pos;
        }
        world.spawn(
            creature::aftik_builder_with_stats(profile, true)
                .add_bundle((CrewMember(crew), iter_pos))
                .build(),
        );
    }

    (controlled, ship)
}

pub fn load_and_deploy_location(
    location_name: &str,
    messages: &mut Messages,
    state: &mut GameState,
) -> Result<(), String> {
    let (gen_context, start_pos) =
        load_location_into_world(location_name, LocationGenContext::clone_from(state))?;
    gen_context.apply_to_game_state(state);

    let ship_exit = state.world.get::<&Ship>(state.ship).unwrap().exit_pos;
    door::place_pair(
        &mut state.world,
        DoorInfo {
            pos: start_pos,
            symbol: Symbol('v'),
            model_id: ModelId::new("ship"),
            kind: DoorKind::Door,
            name: Noun::new("ship", "ships"),
        },
        DoorInfo {
            pos: ship_exit,
            symbol: Symbol('^'),
            model_id: ModelId::new("ship_exit"),
            kind: DoorKind::Door,
            name: Noun::new("ship exit", "ship exits"),
        },
        None,
    );

    deploy_crew_at_new_location(start_pos, state);

    if state.generation_state.is_at_fortuna() {
        messages.add(
            "The ship arrives at the location of the fortuna chest, and the crew exit the ship.",
        )
    } else {
        messages.add("The ship arrives at a new location, and the crew exit the ship.");
    }
    Ok(())
}

struct LocationGenContext {
    world: World,
    character_profiles: Vec<AftikProfile>,
    rng: ThreadRng,
}

impl LocationGenContext {
    fn clone_from(state: &GameState) -> Self {
        Self {
            world: serialization::world::serialize_clone(&state.world)
                .expect("Unexpected error when cloning world"),
            character_profiles: state.generation_state.character_profiles.clone(),
            rng: thread_rng(),
        }
    }

    fn apply_to_game_state(self, game_state: &mut GameState) {
        game_state.world = self.world;
        game_state.generation_state.character_profiles = self.character_profiles;
    }
}

fn load_location_into_world(
    location_name: &str,
    mut gen_context: LocationGenContext,
) -> Result<(LocationGenContext, Pos), String> {
    let start_pos = load_data(location_name)
        .and_then(|location_data| {
            location_data.build(
                &mut gen_context.world,
                &mut gen_context.character_profiles,
                &mut gen_context.rng,
            )
        })
        .map_err(|message| format!("Error loading location {location_name}: {message}"))?;

    let areas_with_aggressive_creatures = gen_context
        .world
        .query::<(&Pos, &Hostile)>()
        .iter()
        .filter(|&(_, (_, hostile))| hostile.aggressive)
        .map(|(_, (pos, _))| pos.get_area())
        .collect::<HashSet<_>>();
    for (_, (pos, hostile)) in gen_context
        .world
        .query_mut::<(&Pos, &mut Hostile)>()
        .into_iter()
    {
        hostile.aggressive |= areas_with_aggressive_creatures.contains(&pos.get_area());
    }

    Ok((gen_context, start_pos))
}

fn deploy_crew_at_new_location(start_pos: Pos, state: &mut GameState) {
    let world = &mut state.world;
    let mut crew_members = world
        .query::<()>()
        .with::<&CrewMember>()
        .iter()
        .map(|pair| pair.0)
        .collect::<Vec<_>>();

    let controlled_index = crew_members
        .iter()
        .position(|&entity| entity == state.controlled)
        .expect("Controlled character should exist and be a crew member");
    crew_members.swap(0, controlled_index);
    let direction = Direction::towards_center(start_pos, world);
    for character in crew_members {
        if let Err(blockage) = position::check_is_pos_blocked(start_pos, world) {
            if blockage.try_push(direction, world).is_err() {
                break;
            }
        }
        world.insert(character, (start_pos, direction)).unwrap();
        let _ = world.remove_one::<Waiting>(character);
    }
}

pub fn load_data(name: &str) -> Result<LocationData, String> {
    let file = File::open(format!("assets/location/{name}.json"))
        .map_err(|error| format!("Failed to open file: {error}"))?;
    serde_json::from_reader(file).map_err(|error| format!("Failed to parse file: {error}"))
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

    let mut buffer = CommandBuffer::new();
    for (entity, keep) in world.query_mut::<Satisfies<&Keep>>() {
        if !keep {
            buffer.despawn(entity);
        } else {
            buffer.remove_one::<Keep>(entity);
        }
    }
    buffer.run_on(world);
}
