use crate::action::{item, Action, CrewMember};
use crate::area::{Locations, PickResult, Ship, ShipStatus};
use crate::command::{CommandResult, Target};
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{Frame, Messages, NameData, StatusCache};
use crate::{action, ai, area, command, status, view};
use hecs::{CommandBuffer, Entity, World};
use rand::prelude::ThreadRng;
use rand::thread_rng;

pub fn setup(locations: Locations) -> Game {
    let mut world = World::new();
    let rng = thread_rng();

    let (controlled, ship) = area::init(&mut world);

    Game {
        world,
        rng,
        controlled,
        has_introduced_controlled: false,
        ship,
        state: State::Prepare,
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
    state: State,
    locations: Locations,
    cache: StatusCache,
}

#[derive(Debug)]
enum State {
    Prepare,
    Load(String),
    Choose(area::Choice),
    AtLocation,
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
            match &self.state {
                State::Choose(_) | State::CommandInput => return Ok(TakeInput),
                State::Prepare => self.prepare_next_location(view_buffer)?,
                State::Load(location) => {
                    area::load_location(
                        &mut self.world,
                        &mut view_buffer.messages,
                        self.ship,
                        location,
                    );
                    if !self.has_introduced_controlled {
                        view_buffer.messages.add(format!(
                            "You're playing as the aftik {}.",
                            NameData::find(&self.world, self.controlled).definite()
                        ));
                        self.has_introduced_controlled = true;
                    }

                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
                    self.state = State::AtLocation;
                }
                State::AtLocation => self.tick(view_buffer)?,
                State::ChangeControlled(character) => {
                    self.change_character(*character, view_buffer);
                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
                    self.state = State::AtLocation;
                }
            }
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Result<(), Messages> {
        match &self.state {
            State::Choose(choice) => {
                let location = self
                    .locations
                    .try_make_choice(choice, input, &mut self.rng)?;
                self.state = State::Load(location);
            }
            State::CommandInput => {
                match command::try_parse_input(input, &self.world, self.controlled)? {
                    CommandResult::Action(action, target) => {
                        insert_action(&mut self.world, self.controlled, action, target);
                        self.state = State::AtLocation;
                    }
                    CommandResult::ChangeControlled(character) => {
                        self.state = State::ChangeControlled(character);
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
            PickResult::Location(location) => self.state = State::Load(location),
            PickResult::Choice(choice) => {
                view_buffer.push_frame(Frame::LocationChoice(choice.present()));
                self.state = State::Choose(choice);
            }
        };
        Ok(())
    }

    fn tick(&mut self, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
        if self.world.get::<&Action>(self.controlled).is_err() {
            self.state = State::CommandInput;
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
                health.restore_to_full();
            }
            self.state = State::Prepare;
        } else {
            view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
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

    action::tick(world, &mut game.rng, &mut view_buffer.messages, controlled);

    detect_low_health(world, &mut view_buffer.messages, controlled);

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
    Ok(())
}

fn is_ship_launching(world: &World, area: Entity) -> bool {
    world
        .get::<&Ship>(area)
        .map(|ship| ship.status == ShipStatus::Launching)
        .unwrap_or(false)
}

struct LowHealth;

fn detect_low_health(world: &mut World, messages: &mut Messages, character: Entity) {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let mut command_buffer = CommandBuffer::new();
    for (entity, (pos, health)) in world.query::<(&Pos, &Health)>().iter() {
        let has_tag = world.get::<&LowHealth>(entity).is_ok();
        let visible_low_health = pos.is_in(area) && health.as_fraction() < 0.5;
        if has_tag && !visible_low_health {
            command_buffer.remove_one::<LowHealth>(entity);
        }
        if !has_tag && visible_low_health && health.is_alive() {
            command_buffer.insert_one(entity, LowHealth);
            if entity != character {
                messages.add(format!(
                    "{} is badly hurt.",
                    NameData::find(world, entity).definite()
                ));
            }
        }
    }
    command_buffer.run_on(world);
}
