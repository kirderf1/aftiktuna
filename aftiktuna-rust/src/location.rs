pub mod generate;

use self::generate::creature;
use self::generate::door::{self, DoorInfo};
use crate::asset::location::{DoorPairData, DoorType, LocationData};
use crate::asset::profile::CharacterProfile;
use crate::asset::{CrewData, GameAssets};
use crate::core::area::{self, FuelAmount, ShipRoom, ShipState, ShipStatus};
use crate::core::behavior::{ObservationTarget, Waiting};
use crate::core::display::{ModelId, SpeciesColorId};
use crate::core::name::NounId;
use crate::core::position::{self, Direction, Pos};
use crate::core::status::Morale;
use crate::core::store::Points;
use crate::core::{CrewMember, Door, DoorKind, inventory};
use crate::game_loop::GameState;
use crate::view::text::{self, Messages};
use crate::{asset, serialization};
use hecs::{CommandBuffer, Entity, Satisfies, World};
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::index;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Serialize, Deserialize)]
enum TrackedState {
    BeforeFortuna { remaining_locations_count: i32 },
    AtFortuna,
}

#[derive(Serialize, Deserialize)]
pub struct GenerationState {
    locations: Locations,
    state: TrackedState,
    aftik_color_names: HashMap<SpeciesColorId, Vec<String>>,
}

impl GenerationState {
    pub fn load_new(locations_before_fortuna: i32) -> Result<Self, asset::Error> {
        Ok(Self {
            locations: Locations::load_from_json()?,
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: locations_before_fortuna,
            },
            aftik_color_names: asset::load_aftik_color_names()?,
        })
    }

    pub fn single(location: String) -> Result<Self, asset::Error> {
        Ok(Self {
            locations: Locations::single(location),
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: 1,
            },
            aftik_color_names: asset::load_aftik_color_names()?,
        })
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

    pub fn locations_before_fortuna(&self) -> i32 {
        match self.state {
            TrackedState::BeforeFortuna {
                remaining_locations_count,
            } => remaining_locations_count,
            TrackedState::AtFortuna => 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Locations {
    categories: Vec<Category>,
    fortuna_locations: Vec<String>,
}

impl Locations {
    pub fn load_from_json() -> Result<Self, asset::Error> {
        asset::load_json_asset("locations.json")
    }

    pub fn all_location_names(&self) -> impl Iterator<Item = &String> {
        self.categories
            .iter()
            .flat_map(|category| category.location_names.iter())
            .chain(self.fortuna_locations.iter())
    }

    fn single(location: String) -> Self {
        Locations {
            categories: vec![Category {
                name: "test".to_string(),
                description: String::default(),
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
            .map(|index| Alternative {
                index,
                name: self.categories[index].name.clone(),
                description: self.categories[index].description.clone(),
            })
            .collect::<Vec<_>>();

        PickResult::Choice(Choice(alternatives))
    }

    fn pick_from_category(&mut self, category_index: usize, rng: &mut impl Rng) -> String {
        let category = self.categories.get_mut(category_index).unwrap();
        let chosen_location = category
            .location_names
            .swap_remove(rng.random_range(0..category.location_names.len()));
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
            .remove(rng.random_range(0..self.fortuna_locations.len()));
        PickResult::Location(location)
    }
}

pub enum PickResult {
    None,
    Location(String),
    Choice(Choice),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Alternative {
    index: usize,
    name: String,
    description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice(Vec<Alternative>);

impl Choice {
    pub fn presentation_text_lines(&self) -> Vec<String> {
        let Choice(alternatives) = self;
        let mut text_lines = Vec::new();
        text_lines.push("On the next planet, there are two destination targets:".to_owned());
        for alternative in alternatives {
            text_lines.push(format!(
                "{}: {}",
                text::capitalize(&alternative.name),
                alternative.description
            ));
        }
        text_lines.push("Pick the location to travel to next.".to_owned());
        text_lines
    }

    pub fn alternatives(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|alternative| alternative.name.clone())
            .collect()
    }
}

impl Choice {
    fn try_choose(&self, input: &str) -> Option<usize> {
        self.0
            .iter()
            .find(|alternative| alternative.name.eq_ignore_ascii_case(input))
            .map(|alternative| alternative.index)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Category {
    name: String,
    description: String,
    pub location_names: Vec<String>,
}

pub struct InitialSpawnData {
    pub world: World,
    pub controlled_character: Entity,
    pub ship_core: Entity,
}

pub(crate) fn spawn_starting_crew_and_ship(
    crew_data: CrewData,
    generation_state: &mut GenerationState,
    assets: &GameAssets,
) -> Result<InitialSpawnData, String> {
    let mut gen_context = LocationGenContext {
        world: World::new(),
        aftik_color_names: generation_state.aftik_color_names.clone(),
        assets,
        rng: rand::rng(),
    };
    let ship_data = LocationData::load_from_json("crew_ship")?;

    let build_data = generate::build_location(ship_data, &mut gen_context)?;
    let LocationGenContext {
        mut world,
        aftik_color_names,
        assets: _,
        mut rng,
    } = gen_context;
    generation_state.aftik_color_names = aftik_color_names;

    let food_deposit_pos = build_data
        .food_deposit_pos
        .ok_or_else(|| "Expected ship location to have food deposit".to_string())?;
    for &room in &build_data.spawned_areas {
        world
            .insert_one(room, ShipRoom)
            .expect("Expected spawned room to exist");
    }

    let ship_core = world.spawn((ShipState {
        status: ShipStatus::NeedFuel(FuelAmount::TwoCans),
        exit_pos: build_data.entry_pos,
        item_pos: food_deposit_pos,
    },));
    let crew = world.spawn((Points(crew_data.points),));

    let mut crew_profiles = Vec::<CharacterProfile>::new();
    for profile in crew_data.crew {
        if let Some(profile) = profile.unwrap(
            &mut generation_state.aftik_color_names,
            &mut rng,
            &crew_profiles
                .iter()
                .map(|profile| &profile.color)
                .collect::<Vec<_>>(),
        ) {
            crew_profiles.push(profile);
        }
    }
    let mut crew_iter = crew_profiles.into_iter();
    let mut iter_pos = Pos::new(build_data.spawned_areas[0], 0, &world);

    let controlled_character = crew_iter
        .next()
        .ok_or_else(|| "Crew must not be empty".to_string())?;
    let controlled_character = world.spawn(
        creature::character_builder_with_stats(
            controlled_character,
            true,
            &assets.species_data_map,
            &mut rng,
        )?
        .add_bundle((CrewMember(crew), iter_pos))
        .build(),
    );

    'add_crew: for profile in crew_iter {
        while position::check_is_pos_blocked(None, iter_pos, &world).is_err() {
            let Some(new_pos) = iter_pos.try_offset(1, &world) else {
                eprintln!(
                    "Tried initializing a crew that is too large. Not all crew members will be added."
                );
                break 'add_crew;
            };
            iter_pos = new_pos;
        }
        world.spawn(
            creature::character_builder_with_stats(
                profile,
                true,
                &assets.species_data_map,
                &mut rng,
            )?
            .add_bundle((CrewMember(crew), iter_pos))
            .build(),
        );
    }

    for (_, morale) in world.query_mut::<&mut Morale>().with::<&CrewMember>() {
        morale.journey_start_effect();
    }

    Ok(InitialSpawnData {
        world,
        controlled_character,
        ship_core,
    })
}

pub(crate) fn setup_location_into_game(
    location_name: &str,
    messages: &mut Messages,
    state: &mut GameState,
    assets: &GameAssets,
) -> Result<(), String> {
    let mut gen_context = LocationGenContext::clone_from(state, assets);

    let build_data = LocationData::load_from_json(location_name)
        .and_then(|location_data| generate::build_location(location_data, &mut gen_context))
        .map_err(|message| format!("Error loading location {location_name}: {message}"))?;

    gen_context.apply_to_game_state(state);

    let ship_exit = state
        .world
        .get::<&ShipState>(state.ship_core)
        .unwrap()
        .exit_pos;
    let (ship_entity, _) = door::place_pair(
        &mut state.world,
        DoorInfo {
            pos: build_data.entry_pos,
            model_id: ModelId::ship(),
            kind: DoorKind::Door,
            noun: NounId::from("ship"),
            adjective: None,
        },
        DoorInfo {
            pos: ship_exit,
            model_id: DoorType::Doorway.into(),
            kind: DoorKind::Door,
            noun: NounId::from("ship_exit"),
            adjective: None,
        },
        &DoorPairData::default(),
    );
    state
        .world
        .insert_one(ship_entity, ObservationTarget)
        .unwrap();

    deploy_crew_at_new_location(build_data.entry_pos, state);

    if state.generation_state.is_at_fortuna() {
        messages.add(
            "The ship arrives at the location of the fortuna chest, and the crew exit the ship.",
        )
    } else {
        messages.add("The ship arrives at a new location, and the crew exit the ship.");
    }
    Ok(())
}

pub struct LocationGenContext<'a> {
    world: World,
    aftik_color_names: HashMap<SpeciesColorId, Vec<String>>,
    assets: &'a GameAssets,
    rng: ThreadRng,
}

impl<'a> LocationGenContext<'a> {
    fn clone_from(state: &GameState, assets: &'a GameAssets) -> Self {
        Self {
            world: serialization::world::serialize_clone(&state.world)
                .expect("Unexpected error when cloning world"),
            aftik_color_names: state.generation_state.aftik_color_names.clone(),
            assets,
            rng: rand::rng(),
        }
    }

    pub fn dummy(assets: &'a GameAssets) -> Self {
        Self {
            world: World::new(),
            aftik_color_names: Default::default(),
            assets,
            rng: rand::rng(),
        }
    }

    fn apply_to_game_state(self, game_state: &mut GameState) {
        game_state.world = self.world;
        game_state.generation_state.aftik_color_names = self.aftik_color_names;
    }
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
        if let Err(blockage) = position::check_is_pos_blocked(Some(character), start_pos, world) {
            let push_result = blockage.try_push(direction, world);
            if push_result.is_err() {
                break;
            }
        }
        world.insert(character, (start_pos, direction)).unwrap();
        let _ = world.remove_one::<Waiting>(character);
    }
}

struct Keep;

pub fn despawn_all_except_ship(world: &mut World) {
    let mut buffer = CommandBuffer::new();
    for (entity, _) in world
        .query::<()>()
        .with::<hecs::Or<&ShipState, &ShipRoom>>()
        .iter()
    {
        buffer.insert_one(entity, Keep);
    }
    for (entity, _) in world
        .query::<&Pos>()
        .iter()
        .filter(|&(_, pos)| area::is_in_ship(*pos, world))
    {
        // Do not preserve the ship exit. It will be respawned with the new location.
        if let Ok(door) = world.get::<&Door>(entity)
            && !area::is_in_ship(door.destination, world)
        {
            continue;
        }

        buffer.insert_one(entity, Keep);
        if let Ok(door) = world.get::<&Door>(entity) {
            buffer.insert_one(door.door_pair, Keep);
        }
        if let Some(item) = inventory::get_wielded(world, entity) {
            buffer.insert_one(item, Keep);
        }
        for item in inventory::get_inventory(world, entity) {
            buffer.insert_one(item, Keep);
        }
        if let Ok(crew) = world
            .get::<&CrewMember>(entity)
            .map(|crew_member| crew_member.0)
        {
            buffer.insert_one(crew, Keep);
        }
    }
    buffer.run_on(world);

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
