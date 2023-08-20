use crate::action::{combat, item, Action, CrewMember, OpenedChest};
use crate::area::{LocationTracker, PickResult, Ship, ShipStatus};
use crate::game_interface::Phase;
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{Frame, Messages, NameData, StatusCache};
use crate::{action, ai, area, serialization, status, view};
use hecs::{Entity, World};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameState {
    #[serde(serialize_with = "serialization::serialize_world")]
    #[serde(deserialize_with = "serialization::deserialize_world")]
    pub world: World,
    #[serde(skip)]
    pub rng: ThreadRng,
    pub locations: LocationTracker,
    pub ship: Entity,
    pub controlled: Entity,
    pub status_cache: StatusCache,
    has_introduced_controlled: bool,
}

pub fn setup(locations: LocationTracker) -> GameState {
    let mut world = World::new();

    let (controlled, ship) = area::init(&mut world);

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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum StopType {
    Win,
    Lose,
}

impl StopType {
    pub fn messages(self) -> Messages {
        match self {
            StopType::Win => Messages::from("Congratulations, you won!"),
            StopType::Lose => Messages::from("You lost."),
        }
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
    serialization::check_world_components(&state.world);

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
            area::load_location(state, &mut view_buffer.messages, &location);
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
    if world.get::<&Action>(controlled).is_err()
        && is_wait_requested(world, controlled)
        && is_safe(world, world.get::<&Pos>(controlled).unwrap().get_area())
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

pub fn is_safe(world: &World, area: Entity) -> bool {
    world
        .query::<&Pos>()
        .with::<&combat::IsFoe>()
        .iter()
        .all(|(_, pos)| !pos.is_in(area))
}

fn tick_and_check(state: &mut GameState, view_buffer: &mut view::Buffer) -> Result<Step, Phase> {
    if should_take_user_input(&state.world, state.controlled) {
        return Err(Phase::CommandInput);
    }

    tick(state, view_buffer);

    if let Err(stop_type) = check_player_state(state, view_buffer) {
        view_buffer.push_ending_frame(&state.world, state.controlled, stop_type);
        return Err(Phase::Stopped(stop_type));
    }

    let area = state
        .world
        .get::<&Pos>(state.controlled)
        .unwrap()
        .get_area();
    if is_ship_launching(&state.world, area) {
        view_buffer
            .messages
            .add("The ship leaves for the next planet.");
        view_buffer.capture_view(state);

        area::despawn_all_except_ship(&mut state.world, state.ship);
        state.world.get::<&mut Ship>(state.ship).unwrap().status = ShipStatus::NeedTwoCans;
        for (_, health) in state.world.query_mut::<&mut Health>() {
            health.restore_fraction(0.5);
        }
        Ok(Step::PrepareNextLocation)
    } else {
        view_buffer.capture_view(state);
        Ok(Step::PrepareTick)
    }
}

fn should_take_user_input(world: &World, controlled: Entity) -> bool {
    world.get::<&Action>(controlled).is_err()
}

fn tick(state: &mut GameState, view_buffer: &mut view::Buffer) {
    ai::tick(&mut state.world);

    action::tick(state, &mut view_buffer.messages);

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

    for aftik in dead_crew {
        item::drop_all_items(&mut state.world, aftik);
        state.world.despawn(aftik).unwrap();
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

fn is_ship_launching(world: &World, area: Entity) -> bool {
    world
        .get::<&Ship>(area)
        .map(|ship| ship.status == ShipStatus::Launching)
        .unwrap_or(false)
}

fn change_character(state: &mut GameState, character: Entity, view_buffer: &mut view::Buffer) {
    state.controlled = character;

    view_buffer.messages.add(format!(
        "You're now playing as the aftik {}.",
        NameData::find(&state.world, state.controlled).definite()
    ));
}
