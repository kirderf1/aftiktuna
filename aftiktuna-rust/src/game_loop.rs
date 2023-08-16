use crate::action::{combat, item, Action, CrewMember, OpenedChest};
use crate::area::{LocationTracker, PickResult, Ship, ShipStatus};
use crate::command::{CommandResult, Target};
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{Frame, Messages, NameData, StatusCache};
use crate::{action, ai, area, command, status, view};
use hecs::{Entity, World};
use rand::prelude::ThreadRng;
use rand::thread_rng;

pub fn setup(locations: LocationTracker) -> Game {
    let mut world = World::new();
    let rng = thread_rng();

    let (controlled, ship) = area::init(&mut world);

    Game {
        world,
        rng,
        state: GameState {
            phase: Phase::Introduce,
            locations,
            ship,
            controlled,
            status_cache: StatusCache::default(),
            has_introduced_controlled: false,
        },
    }
}

pub struct TakeInput;

#[derive(Copy, Clone)]
pub enum StopType {
    Win,
    Lose,
}

pub struct Game {
    world: World,
    rng: ThreadRng,
    state: GameState,
}

struct GameState {
    phase: Phase,
    locations: LocationTracker,
    ship: Entity,
    controlled: Entity,
    status_cache: StatusCache,
    has_introduced_controlled: bool,
}

#[derive(Debug)]
enum Phase {
    Introduce,
    PrepareNextLocation,
    LoadLocation(String),
    ChooseLocation(area::Choice),
    PrepareTick,
    Tick,
    CommandInput,
    ChangeControlled(Entity),
}

impl StopType {
    pub fn messages(self) -> Messages {
        match self {
            StopType::Win => Messages::from("Congratulations, you won!"),
            StopType::Lose => Messages::from("You lost."),
        }
    }
}

impl Game {
    pub fn run(&mut self) -> (Result<TakeInput, StopType>, view::Buffer) {
        let mut buffer = Default::default();
        let result = self.run_with_buffer(&mut buffer);
        if let Err(stop_type) = result {
            buffer.push_frame(Frame::Ending(stop_type));
        }
        (result, buffer)
    }

    fn run_with_buffer(&mut self, view_buffer: &mut view::Buffer) -> Result<TakeInput, StopType> {
        loop {
            match &self.state.phase {
                Phase::Introduce => {
                    view_buffer.push_frame(Frame::Introduction);
                    self.state.phase = Phase::PrepareNextLocation;
                }
                Phase::ChooseLocation(_) | Phase::CommandInput => return Ok(TakeInput),
                Phase::PrepareNextLocation => prepare_next_location(self, view_buffer)?,
                Phase::LoadLocation(location) => {
                    area::load_location(
                        &mut self.world,
                        &mut view_buffer.messages,
                        self.state.ship,
                        location,
                        self.state.locations.is_at_fortuna(),
                    );
                    if !self.state.has_introduced_controlled {
                        view_buffer.messages.add(format!(
                            "You're playing as the aftik {}.",
                            NameData::find(&self.world, self.state.controlled).definite()
                        ));
                        self.state.has_introduced_controlled = true;
                    }

                    view_buffer.capture_view(
                        &self.world,
                        self.state.controlled,
                        &mut self.state.status_cache,
                    );
                    self.state.phase = Phase::PrepareTick;
                }
                Phase::PrepareTick => {
                    ai::prepare_intentions(&mut self.world);
                    insert_wait_if_relevant(&mut self.world, self.state.controlled);
                    self.state.phase = Phase::Tick;
                }
                Phase::Tick => tick_and_check(self, view_buffer)?,
                Phase::ChangeControlled(character) => {
                    let character = *character;
                    change_character(&self.world, &mut self.state, character, view_buffer);
                    view_buffer.capture_view(
                        &self.world,
                        self.state.controlled,
                        &mut self.state.status_cache,
                    );
                    self.state.phase = Phase::Tick;
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Result<(), Messages> {
        match &self.state.phase {
            Phase::ChooseLocation(choice) => {
                let location =
                    self.state
                        .locations
                        .try_make_choice(choice, input, &mut self.rng)?;
                self.state.phase = Phase::LoadLocation(location);
            }
            Phase::CommandInput => {
                match command::try_parse_input(
                    input,
                    &self.world,
                    self.state.controlled,
                    self.state.locations.is_at_fortuna(),
                )? {
                    CommandResult::Action(action, target) => {
                        insert_action(&mut self.world, self.state.controlled, action, target);
                        self.state.phase = Phase::Tick;
                    }
                    CommandResult::ChangeControlled(character) => {
                        self.state.phase = Phase::ChangeControlled(character);
                    }
                    CommandResult::Info(messages) => return Err(messages),
                }
            }
            state => panic!("Handling input in unexpected state {state:?}"),
        }
        Ok(())
    }
}

fn prepare_next_location(game: &mut Game, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
    match game.state.locations.next(&mut game.rng) {
        PickResult::None => return Err(StopType::Win),
        PickResult::Location(location) => game.state.phase = Phase::LoadLocation(location),
        PickResult::Choice(choice) => {
            view_buffer.push_frame(Frame::LocationChoice(choice.clone()));
            game.state.phase = Phase::ChooseLocation(choice);
        }
    };
    Ok(())
}

fn tick_and_check(game: &mut Game, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
    if should_take_user_input(&game.world, game.state.controlled) {
        game.state.phase = Phase::CommandInput;
        return Ok(());
    }

    tick(game, view_buffer);

    check_player_state(&game.world, &mut game.state, view_buffer)?;

    let area = game
        .world
        .get::<&Pos>(game.state.controlled)
        .unwrap()
        .get_area();
    if is_ship_launching(&game.world, area) {
        view_buffer
            .messages
            .add("The ship leaves for the next planet.");
        view_buffer.capture_view(
            &game.world,
            game.state.controlled,
            &mut game.state.status_cache,
        );

        area::despawn_all_except_ship(&mut game.world, game.state.ship);
        game.world.get::<&mut Ship>(game.state.ship).unwrap().status = ShipStatus::NeedTwoCans;
        for (_, health) in game.world.query_mut::<&mut Health>() {
            health.restore_fraction(0.5);
        }
        game.state.phase = Phase::PrepareNextLocation;
    } else {
        view_buffer.capture_view(
            &game.world,
            game.state.controlled,
            &mut game.state.status_cache,
        );
        game.state.phase = Phase::PrepareTick;
    }
    Ok(())
}

fn tick(game: &mut Game, view_buffer: &mut view::Buffer) {
    let world = &mut game.world;
    let controlled = game.state.controlled;

    ai::tick(world);

    action::tick(
        world,
        &mut game.rng,
        &mut view_buffer.messages,
        controlled,
        game.state.locations.is_at_fortuna(),
    );

    status::detect_low_health(world, &mut view_buffer.messages, controlled);
    status::detect_low_stamina(world, &mut view_buffer.messages, controlled);

    handle_aftik_deaths(world, view_buffer, controlled);

    for (_, stamina) in world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }
}

fn change_character(
    world: &World,
    state: &mut GameState,
    character: Entity,
    view_buffer: &mut view::Buffer,
) {
    state.controlled = character;

    view_buffer.messages.add(format!(
        "You're now playing as the aftik {}.",
        NameData::find(world, state.controlled).definite()
    ));
}

fn insert_action(world: &mut World, controlled: Entity, action: Action, target: Target) {
    match target {
        Target::Controlled => {
            world.insert_one(controlled, action).unwrap();
        }
        Target::Crew => {
            let area = world.get::<&Pos>(controlled).unwrap().get_area();
            let aftiks = world
                .query::<&Pos>()
                .with::<&CrewMember>()
                .iter()
                .filter(|(_, pos)| pos.is_in(area))
                .map(|(aftik, _)| aftik)
                .collect::<Vec<_>>();
            for aftik in aftiks {
                world.insert_one(aftik, action.clone()).unwrap();
            }
        }
    }
}

fn handle_aftik_deaths(
    world: &mut World,
    view_buffer: &mut view::Buffer,
    controlled_aftik: Entity,
) {
    let dead_crew = world
        .query::<&Health>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(_, health)| health.is_dead())
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();

    for &aftik in &dead_crew {
        view_buffer.messages.add(format!(
            "{} is dead.",
            NameData::find(world, aftik).definite()
        ));
    }

    if !status::is_alive(controlled_aftik, world) {
        view_buffer.capture_view(world, controlled_aftik, &mut StatusCache::default());
    }

    for aftik in dead_crew {
        item::drop_all_items(world, aftik);
        world.despawn(aftik).unwrap();
    }
}

fn check_player_state(
    world: &World,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<(), StopType> {
    if world.get::<&CrewMember>(state.controlled).is_err() {
        let (next_character, _) = world
            .query::<()>()
            .with::<&CrewMember>()
            .iter()
            .next()
            .ok_or(StopType::Lose)?;
        change_character(world, state, next_character, view_buffer);
    }

    if world.get::<&OpenedChest>(state.controlled).is_ok() {
        view_buffer.capture_view(world, state.controlled, &mut state.status_cache);
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

fn should_take_user_input(world: &World, controlled: Entity) -> bool {
    world.get::<&Action>(controlled).is_err()
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

fn insert_wait_if_relevant(world: &mut World, controlled: Entity) {
    if world.get::<&Action>(controlled).is_err()
        && is_wait_requested(world, controlled)
        && is_safe(world, world.get::<&Pos>(controlled).unwrap().get_area())
    {
        world.insert_one(controlled, Action::Wait).unwrap();
    }
}
