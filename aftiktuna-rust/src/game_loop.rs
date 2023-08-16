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
        controlled,
        has_introduced_controlled: false,
        ship,
        phase: Phase::Introduce,
        locations,
        cache: StatusCache::default(),
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
    controlled: Entity,
    has_introduced_controlled: bool,
    ship: Entity,
    phase: Phase,
    locations: LocationTracker,
    cache: StatusCache,
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

    pub fn run_with_buffer(
        &mut self,
        view_buffer: &mut view::Buffer,
    ) -> Result<TakeInput, StopType> {
        loop {
            match &self.phase {
                Phase::Introduce => {
                    view_buffer.push_frame(Frame::Introduction);
                    self.phase = Phase::PrepareNextLocation;
                }
                Phase::ChooseLocation(_) | Phase::CommandInput => return Ok(TakeInput),
                Phase::PrepareNextLocation => self.prepare_next_location(view_buffer)?,
                Phase::LoadLocation(location) => {
                    area::load_location(
                        &mut self.world,
                        &mut view_buffer.messages,
                        self.ship,
                        location,
                        self.locations.is_at_fortuna(),
                    );
                    if !self.has_introduced_controlled {
                        view_buffer.messages.add(format!(
                            "You're playing as the aftik {}.",
                            NameData::find(&self.world, self.controlled).definite()
                        ));
                        self.has_introduced_controlled = true;
                    }

                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
                    self.phase = Phase::PrepareTick;
                }
                Phase::PrepareTick => {
                    ai::prepare_intentions(&mut self.world);
                    insert_wait_if_relevant(&mut self.world, self.controlled);
                    self.phase = Phase::Tick;
                }
                Phase::Tick => self.tick(view_buffer)?,
                Phase::ChangeControlled(character) => {
                    self.change_character(*character, view_buffer);
                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
                    self.phase = Phase::Tick;
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Result<(), Messages> {
        match &self.phase {
            Phase::ChooseLocation(choice) => {
                let location = self
                    .locations
                    .try_make_choice(choice, input, &mut self.rng)?;
                self.phase = Phase::LoadLocation(location);
            }
            Phase::CommandInput => {
                match command::try_parse_input(
                    input,
                    &self.world,
                    self.controlled,
                    self.locations.is_at_fortuna(),
                )? {
                    CommandResult::Action(action, target) => {
                        insert_action(&mut self.world, self.controlled, action, target);
                        self.phase = Phase::Tick;
                    }
                    CommandResult::ChangeControlled(character) => {
                        self.phase = Phase::ChangeControlled(character);
                    }
                    CommandResult::Info(messages) => return Err(messages),
                }
            }
            state => panic!("Handling input in unexpected state {state:?}"),
        }
        Ok(())
    }

    fn prepare_next_location(&mut self, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
        match self.locations.next(&mut self.rng) {
            PickResult::None => return Err(StopType::Win),
            PickResult::Location(location) => self.phase = Phase::LoadLocation(location),
            PickResult::Choice(choice) => {
                view_buffer.push_frame(Frame::LocationChoice(choice.clone()));
                self.phase = Phase::ChooseLocation(choice);
            }
        };
        Ok(())
    }

    fn tick(&mut self, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
        if should_take_user_input(&self.world, self.controlled) {
            self.phase = Phase::CommandInput;
            return Ok(());
        }

        tick(self, view_buffer);

        check_player_state(self, view_buffer)?;

        let area = self.world.get::<&Pos>(self.controlled).unwrap().get_area();
        if is_ship_launching(&self.world, area) {
            view_buffer
                .messages
                .add("The ship leaves for the next planet.");
            view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);

            area::despawn_all_except_ship(&mut self.world, self.ship);
            self.world.get::<&mut Ship>(self.ship).unwrap().status = ShipStatus::NeedTwoCans;
            for (_, health) in self.world.query_mut::<&mut Health>() {
                health.restore_fraction(0.5);
            }
            self.phase = Phase::PrepareNextLocation;
        } else {
            view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
            self.phase = Phase::PrepareTick;
        }
        Ok(())
    }

    fn change_character(&mut self, character: Entity, view_buffer: &mut view::Buffer) {
        self.controlled = character;

        view_buffer.messages.add(format!(
            "You're now playing as the aftik {}.",
            NameData::find(&self.world, self.controlled).definite()
        ));
    }
}

fn tick(game: &mut Game, view_buffer: &mut view::Buffer) {
    let world = &mut game.world;
    let controlled = game.controlled;

    ai::tick(world);

    action::tick(
        world,
        &mut game.rng,
        &mut view_buffer.messages,
        controlled,
        game.locations.is_at_fortuna(),
    );

    status::detect_low_health(world, &mut view_buffer.messages, controlled);
    status::detect_low_stamina(world, &mut view_buffer.messages, controlled);

    handle_aftik_deaths(world, view_buffer, controlled);

    for (_, stamina) in world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }
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

fn check_player_state(game: &mut Game, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
    if game.world.get::<&CrewMember>(game.controlled).is_err() {
        let (next_character, _) = game
            .world
            .query::<()>()
            .with::<&CrewMember>()
            .iter()
            .next()
            .ok_or(StopType::Lose)?;
        game.change_character(next_character, view_buffer);
    }

    if game.world.get::<&OpenedChest>(game.controlled).is_ok() {
        view_buffer.capture_view(&game.world, game.controlled, &mut game.cache);
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
