pub mod generate;

use self::generate::creature;
use self::generate::door::{self, DoorInfo};
use crate::asset::location::{DoorPairData, DoorType, LocationData};
use crate::asset::{AftikProfile, CrewData};
use crate::core::area::{Area, BackgroundId, FuelAmount, Ship, ShipControls, ShipStatus};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::{self, Direction, Pos};
use crate::core::store::Points;
use crate::core::{CrewMember, Door, DoorKind, ObservationTarget, Waiting, inventory, item};
use crate::game_loop::GameState;
use crate::view::text::{self, Messages};
use crate::{asset, serialization};
use hecs::{CommandBuffer, Entity, Satisfies, World};
use rand::rngs::ThreadRng;
use rand::seq::index;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize, Deserialize)]
enum TrackedState {
    BeforeFortuna { remaining_locations_count: i32 },
    AtFortuna,
}

#[derive(Serialize, Deserialize)]
pub struct GenerationState {
    locations: Locations,
    state: TrackedState,
    character_profiles: Vec<AftikProfile>,
}

impl GenerationState {
    pub fn load_new(locations_before_fortuna: i32) -> Result<Self, String> {
        Ok(Self {
            locations: Locations::load_from_json()?,
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: locations_before_fortuna,
            },
            character_profiles: asset::load_character_profiles()?,
        })
    }

    pub fn single(location: String) -> Result<Self, String> {
        Ok(Self {
            locations: Locations::single(location),
            state: TrackedState::BeforeFortuna {
                remaining_locations_count: 1,
            },
            character_profiles: asset::load_character_profiles()?,
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
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Locations {
    categories: Vec<Category>,
    fortuna_locations: Vec<String>,
}

impl Locations {
    pub fn load_from_json() -> Result<Self, String> {
        asset::load_json_simple("locations.json")
            .map_err(|message| format!("Error loading \"locations.json\": {message}"))
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

pub(crate) fn spawn_starting_crew_and_ship(
    world: &mut World,
    crew_data: CrewData,
    generation_state: &mut GenerationState,
    rng: &mut impl Rng,
) -> (Entity, Entity) {
    let ship = world.spawn((Area {
        label: "Ship".to_string(),
        size: 5,
        background: BackgroundId::new("ship"),
        background_offset: 0,
        extra_background_layers: Vec::default(),
        darkness: 0.,
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
                eprintln!(
                    "Tried initializing a crew that is too large. Not all crew members will be added."
                );
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

pub fn setup_location_into_game(
    location_name: &str,
    messages: &mut Messages,
    state: &mut GameState,
) -> Result<(), String> {
    let mut gen_context = LocationGenContext::clone_from(state);

    let start_pos = LocationData::load_from_json(location_name)
        .and_then(|location_data| generate::build_location(location_data, &mut gen_context))
        .map_err(|message| format!("Error loading location {location_name}: {message}"))?;

    gen_context.apply_to_game_state(state);

    let ship_exit = state.world.get::<&Ship>(state.ship).unwrap().exit_pos;
    let (ship_entity, _) = door::place_pair(
        &mut state.world,
        DoorInfo {
            pos: start_pos,
            symbol: Symbol('v'),
            model_id: ModelId::ship(),
            kind: DoorKind::Door,
            name: Noun::new("ship", "ships"),
        },
        DoorInfo {
            pos: ship_exit,
            symbol: Symbol('^'),
            model_id: DoorType::Doorway.into(),
            kind: DoorKind::Door,
            name: Noun::new("ship exit", "ship exits"),
        },
        &DoorPairData::default(),
    );
    state
        .world
        .insert_one(ship_entity, ObservationTarget)
        .unwrap();

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

#[derive(Default)]
pub struct LocationGenContext {
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
