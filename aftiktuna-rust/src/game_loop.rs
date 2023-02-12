use crate::action::{item, Action, CrewMember};
use crate::area::{Locations, PickResult, Ship, ShipStatus};
use crate::command::{CommandResult, Target};
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{Messages, NameData, StatusCache};
use crate::{action, ai, area, command, status, view};
use hecs::{CommandBuffer, Entity, World};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use std::io::Write;
use std::{io, mem};

pub fn run(locations: Locations) {
    let mut world = World::new();
    let rng = thread_rng();

    let (controlled, ship) = area::init(&mut world);

    println!(
        "You're playing as the aftik {}.",
        NameData::find(&world, controlled).definite()
    );

    let mut game = Game {
        world,
        rng,
        controlled,
        ship,
        state: State::Prepare,
        locations,
        cache: StatusCache::default(),
    };

    let mut view_buffer = view::Buffer::default();
    match game.run(&mut view_buffer) {
        StopType::Win => {
            view_buffer.print();
            println!();
            println!("Congratulations, you won!");
        }
        StopType::Lose => {
            view_buffer.print();
            println!();
            println!("You lost.");
        }
    }
}

struct Game {
    world: World,
    rng: ThreadRng,
    controlled: Entity,
    ship: Entity,
    state: State,
    locations: Locations,
    cache: StatusCache,
}

enum State {
    Prepare,
    Load(String),
    Choose(area::Choice),
    AtLocation,
}

enum StopType {
    Win,
    Lose,
}

impl Game {
    fn run(&mut self, view_buffer: &mut view::Buffer) -> StopType {
        loop {
            let result = match &self.state {
                State::Prepare => self.prepare_next_location(view_buffer),
                State::Load(location) => {
                    area::load_location(
                        &mut self.world,
                        &mut view_buffer.messages,
                        self.ship,
                        location,
                    );
                    self.state = State::AtLocation;
                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
                    Ok(())
                }
                State::Choose(choice) => {
                    if let Some(location) =
                        self.locations
                            .try_make_choice(choice, read_input(), &mut self.rng)
                    {
                        self.state = State::Load(location);
                    }
                    Ok(())
                }
                State::AtLocation => self.tick(view_buffer),
            };
            if let Err(stop_type) = result {
                return stop_type;
            }
        }
    }

    fn prepare_next_location(&mut self, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
        match self.locations.next(&mut self.rng) {
            PickResult::None => return Err(StopType::Win),
            PickResult::Location(location) => self.state = State::Load(location),
            PickResult::Choice(choice) => {
                view_buffer.push_messages(choice.present());
                mem::take(view_buffer).print();
                self.state = State::Choose(choice);
            }
        };
        Ok(())
    }

    fn tick(&mut self, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
        decision_phase(self, view_buffer);

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

fn decision_phase(game: &mut Game, view_buffer: &mut view::Buffer) {
    while game.world.get::<&Action>(game.controlled).is_err() {
        mem::take(view_buffer).print();
        match parse_user_action(&game.world, game.controlled) {
            InputResult::Action(action, target) => match target {
                Target::Controlled => game.world.insert_one(game.controlled, action).unwrap(),
                Target::Crew => {
                    let area = game.world.get::<&Pos>(game.controlled).unwrap().get_area();
                    insert_crew_action(&mut game.world, area, action);
                }
            },
            InputResult::ChangeControlled(character) => {
                game.change_character(character, view_buffer);
                view_buffer.capture_view(&game.world, game.controlled, &mut game.cache);
            }
        }
    }
}

enum InputResult {
    Action(Action, Target),
    ChangeControlled(Entity),
}

fn parse_user_action(world: &World, controlled: Entity) -> InputResult {
    loop {
        let input = read_input().to_lowercase();

        match command::try_parse_input(&input, world, controlled) {
            Ok(CommandResult::Action(action, target)) => {
                return InputResult::Action(action, target);
            }
            Ok(CommandResult::ChangeControlled(new_aftik)) => {
                return InputResult::ChangeControlled(new_aftik);
            }
            Ok(CommandResult::None) => {}
            Err(message) => println!("{}", view::capitalize(message)),
        }
    }
}

fn insert_crew_action(world: &mut World, area: Entity, action: Action) {
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

fn read_input() -> String {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    String::from(input.trim())
}
