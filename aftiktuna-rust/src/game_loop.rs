use hecs::{Entity, Or, World};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::action::{self, Action};
use crate::ai;
use crate::core::area::{FuelAmount, Ship, ShipStatus};
use crate::core::inventory::Held;
use crate::core::item::FoodRation;
use crate::core::name::{NameData, NameQuery};
use crate::core::position::{Direction, Pos};
use crate::core::status::{Health, Stamina};
use crate::core::{self, inventory, status, CrewMember, OpenedChest, OrderWeight, RepeatingAction};
use crate::game_interface::Phase;
use crate::location::{self, LocationTracker, PickResult};
use crate::serialization;
use crate::view::{self, Frame, Messages, StatusCache};

#[derive(Serialize, Deserialize)]
pub struct GameState {
    #[serde(with = "serialization::world")]
    pub world: World,
    #[serde(skip)]
    pub rng: ThreadRng,
    pub locations: LocationTracker,
    pub ship: Entity,
    pub controlled: Entity,
    pub status_cache: StatusCache,
    pub has_introduced_controlled: bool,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum StopType {
    Win,
    Lose,
}

pub fn setup(locations: LocationTracker) -> GameState {
    let mut world = World::new();

    let (controlled, ship) = location::init(&mut world);

    GameState {
        world,
        rng: thread_rng(),
        locations,
        ship,
        controlled,
        status_cache: StatusCache::default(),
        has_introduced_controlled: false,
    }
}

#[derive(Debug)]
pub enum Step {
    PrepareNextLocation,
    LoadLocation(String),
    PrepareTick,
    Tick,
    ChangeControlled(Entity),
}

pub fn run(mut step: Step, state: &mut GameState) -> (Phase, Vec<Frame>) {
    let mut view_buffer = view::Buffer::default();
    let phase = loop {
        let result = run_step(step, state, &mut view_buffer);
        match result {
            Ok(next_step) => step = next_step,
            Err(phase) => break phase,
        }
    };

    #[cfg(feature = "debug_logging")]
    crate::serialization::check_world_components(&state.world);

    (phase, view_buffer.into_frames())
}

fn run_step(
    phase: Step,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<Step, Phase> {
    match phase {
        Step::PrepareNextLocation => prepare_next_location(state, view_buffer),
        Step::LoadLocation(location) => {
            location::load_location(state, &mut view_buffer.messages, &location);
            if !state.has_introduced_controlled {
                view_buffer.messages.add(format!(
                    "You're playing as the aftik {}.",
                    NameData::find(&state.world, state.controlled).definite()
                ));
                state.has_introduced_controlled = true;
            }

            view_buffer.capture_view(state);
            Ok(Step::PrepareTick)
        }
        Step::PrepareTick => {
            ai::prepare_intentions(&mut state.world);
            insert_wait_if_relevant(&mut state.world, state.controlled);
            Ok(Step::Tick)
        }
        Step::Tick => tick_and_check(state, view_buffer),
        Step::ChangeControlled(character) => {
            change_character(state, character, view_buffer);
            view_buffer.capture_view(state);
            Ok(Step::Tick)
        }
    }
}

fn prepare_next_location(
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<Step, Phase> {
    match state.locations.next(&mut state.rng) {
        PickResult::None => {
            view_buffer.push_ending_frame(&state.world, state.controlled, StopType::Win);
            Err(Phase::Stopped(StopType::Win))
        }
        PickResult::Location(location) => Ok(Step::LoadLocation(location)),
        PickResult::Choice(choice) => {
            view_buffer.push_frame(Frame::LocationChoice(choice.clone()));
            Err(Phase::ChooseLocation(choice))
        }
    }
}

fn insert_wait_if_relevant(world: &mut World, controlled: Entity) {
    if !world
        .satisfies::<Or<&Action, &RepeatingAction>>(controlled)
        .unwrap()
        && is_wait_requested(world, controlled)
        && core::is_safe(world, world.get::<&Pos>(controlled).unwrap().get_area())
    {
        world.insert_one(controlled, Action::Wait).unwrap();
    }
}

fn is_wait_requested(world: &World, controlled: Entity) -> bool {
    let area = world.get::<&Pos>(controlled).unwrap().get_area();
    world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(entity, pos)| *entity != controlled && pos.is_in(area))
        .any(|(entity, _)| ai::is_requesting_wait(world, entity))
}

fn tick_and_check(state: &mut GameState, view_buffer: &mut view::Buffer) -> Result<Step, Phase> {
    if should_take_user_input(&state.world, state.controlled) {
        view_buffer.capture_view(state);
        return Err(Phase::CommandInput);
    }

    let prev_area = state
        .world
        .get::<&Pos>(state.controlled)
        .unwrap()
        .get_area();
    tick(state, view_buffer);

    if let Err(stop_type) = check_player_state(state, view_buffer) {
        view_buffer.push_ending_frame(&state.world, state.controlled, stop_type);
        return Err(Phase::Stopped(stop_type));
    }

    handle_was_waiting(state, view_buffer);

    let area = state
        .world
        .get::<&Pos>(state.controlled)
        .unwrap()
        .get_area();
    if is_ship_launching(&state.world, area) {
        leave_location(state, view_buffer);
        Ok(Step::PrepareNextLocation)
    } else {
        if area != prev_area {
            view_buffer.capture_view(state);
        }
        Ok(Step::PrepareTick)
    }
}

fn should_take_user_input(world: &World, controlled: Entity) -> bool {
    !world
        .satisfies::<Or<&Action, &RepeatingAction>>(controlled)
        .unwrap()
}

fn tick(state: &mut GameState, view_buffer: &mut view::Buffer) {
    ai::tick(&mut state.world);

    action::tick(state, view_buffer);

    status::detect_low_health(
        &mut state.world,
        &mut view_buffer.messages,
        state.controlled,
    );
    status::detect_low_stamina(
        &mut state.world,
        &mut view_buffer.messages,
        state.controlled,
    );

    handle_aftik_deaths(state, view_buffer);

    for (_, stamina) in state.world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }
}

fn handle_aftik_deaths(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let dead_crew = state
        .world
        .query::<&Health>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(_, health)| health.is_dead())
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();

    for &aftik in &dead_crew {
        view_buffer.messages.add(format!(
            "{} is dead.",
            NameData::find(&state.world, aftik).definite()
        ));
    }

    if !status::is_alive(state.controlled, &state.world) {
        state.status_cache = StatusCache::default();
        view_buffer.capture_view(state);
    }

    for character in dead_crew {
        inventory::drop_all_items(&mut state.world, character);
        state.world.remove_one::<CrewMember>(character).unwrap();
    }
}

fn check_player_state(
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<(), StopType> {
    if state.world.get::<&CrewMember>(state.controlled).is_err() {
        let (next_character, _) = state
            .world
            .query::<()>()
            .with::<&CrewMember>()
            .iter()
            .next()
            .ok_or(StopType::Lose)?;
        change_character(state, next_character, view_buffer);
    }

    if state.world.get::<&OpenedChest>(state.controlled).is_ok() {
        view_buffer.capture_view(state);
        return Err(StopType::Win);
    }

    Ok(())
}

fn handle_was_waiting(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let player_pos = *state.world.get::<&Pos>(state.controlled).unwrap();

    let entities = state
        .world
        .query::<()>()
        .with::<&action::WasWaiting>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in entities {
        let pos = *state.world.get::<&Pos>(entity).unwrap();
        if pos.is_in(player_pos.get_area()) && status::is_alive(entity, &state.world) {
            if state
                .world
                .get::<&core::Hostile>(entity)
                .map_or(false, |hostile| !hostile.aggressive)
            {
                state
                    .world
                    .insert_one(entity, Direction::between(pos, player_pos))
                    .unwrap();
                view_buffer.messages.add(format!(
                    "{} makes a threatening pose.",
                    NameData::find(&state.world, entity).definite()
                ));
            }

            if state
                .world
                .get::<&core::Hostile>(entity)
                .map_or(false, |hostile| hostile.aggressive)
            {
                state
                    .world
                    .insert_one(entity, Direction::between(pos, player_pos))
                    .unwrap();
                view_buffer.messages.add(format!(
                    "{} moves in to attack.",
                    NameData::find(&state.world, entity).definite()
                ));
            }
        }
        state
            .world
            .remove_one::<action::WasWaiting>(entity)
            .unwrap();
    }
}

fn is_ship_launching(world: &World, area: Entity) -> bool {
    world
        .get::<&Ship>(area)
        .map(|ship| ship.status == ShipStatus::Launching)
        .unwrap_or(false)
}

fn leave_location(state: &mut GameState, view_buffer: &mut view::Buffer) {
    deposit_items_to_ship(state);

    view_buffer
        .messages
        .add("The ship leaves for the next planet.");

    for (_, (_, query)) in state
        .world
        .query::<(&Pos, NameQuery)>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(_, (pos, _))| !pos.is_in(state.ship))
    {
        let name = NameData::from(query).definite();
        view_buffer.messages.add(format!("{name} was left behind."));
    }
    consume_rations_healing(state, &mut view_buffer.messages);

    view_buffer.capture_view(state);

    location::despawn_all_except_ship(&mut state.world, state.ship);
    state.world.get::<&mut Ship>(state.ship).unwrap().status =
        ShipStatus::NeedFuel(FuelAmount::TwoCans);
}

fn deposit_items_to_ship(state: &mut GameState) {
    let crew_in_ship = state
        .world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(state.ship))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let items = state
        .world
        .query::<&Held>()
        .with::<&FoodRation>()
        .iter()
        .filter(|&(_, held)| crew_in_ship.iter().any(|&entity| held.held_by(entity)))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let item_pos = state.world.get::<&Ship>(state.ship).unwrap().item_pos;
    for item in items {
        state.world.exchange_one::<Held, _>(item, item_pos).unwrap();
    }
}

fn consume_rations_healing(state: &mut GameState, messages: &mut Messages) {
    let mut crew_candidates = state
        .world
        .query::<&Health>()
        .with::<&CrewMember>()
        .iter()
        .filter(|&(_, health)| health.is_hurt())
        .map(|(entity, health)| (entity, health.as_fraction()))
        .collect::<Vec<_>>();
    crew_candidates.sort_by(|(_, a), (_, b)| a.total_cmp(b));
    let mut crew_eating_rations = Vec::new();

    for (crew_candidate, _) in crew_candidates {
        let ration = state
            .world
            .query::<&Pos>()
            .with::<&FoodRation>()
            .iter()
            .find(|&(_, pos)| pos.is_in(state.ship))
            .map(|(entity, _)| entity);
        if let Some(ration) = ration {
            state.world.despawn(ration).unwrap();
            state
                .world
                .get::<&mut Health>(crew_candidate)
                .unwrap()
                .restore_fraction(0.33);
            crew_eating_rations.push(crew_candidate);
        } else {
            break;
        }
    }
    let crew_eating_rations = crew_eating_rations
        .into_iter()
        .map(|entity| NameData::find(&state.world, entity).definite())
        .collect::<Vec<_>>();
    if !crew_eating_rations.is_empty() {
        if let [name] = &crew_eating_rations[..] {
            messages.add(format!("{name} ate a food ration to recover some health."));
        } else {
            let names = crew_eating_rations.join(" and ");
            messages.add(format!(
                "{names} ate a food ration each to recover some health.",
            ));
        }
    }
}

fn change_character(state: &mut GameState, character: Entity, view_buffer: &mut view::Buffer) {
    let _ = state
        .world
        .insert_one(state.controlled, OrderWeight::Creature);
    state
        .world
        .insert_one(character, OrderWeight::Controlled)
        .unwrap();
    state.controlled = character;

    view_buffer.messages.add(format!(
        "You're now playing as the aftik {}.",
        NameData::find(&state.world, character).definite()
    ));
}
