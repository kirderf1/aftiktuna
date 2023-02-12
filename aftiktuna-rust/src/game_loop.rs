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
        load_location: true,
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
    load_location: bool,
    locations: Locations,
    cache: StatusCache,
}

impl Game {
    fn run(&mut self, view_buffer: &mut view::Buffer) -> StopType {
        loop {
            if self.load_location {
                self.load_location = false;
                let location = match self.locations.next(&mut self.rng) {
                    PickResult::None => return StopType::Win,
                    PickResult::Location(location) => location,
                    PickResult::Choice(choice) => {
                        view_buffer.push_messages(choice.present());
                        mem::take(view_buffer).print();

                        loop {
                            if let Some(location) =
                                self.locations
                                    .try_make_choice(&choice, read_input(), &mut self.rng)
                            {
                                break location;
                            }
                        }
                    }
                };
                area::load_location(
                    &mut self.world,
                    &mut view_buffer.messages,
                    self.ship,
                    &location,
                );
                view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
            } else {
                if let Err(stop_type) = tick(self, view_buffer) {
                    match stop_type {
                        TickInterrupt::Lose => return StopType::Lose,
                        TickInterrupt::ControlCharacter(character) => {
                            self.controlled = character;

                            view_buffer.messages.add(format!(
                                "You're now playing as the aftik {}.",
                                NameData::find(&self.world, self.controlled).definite()
                            ));
                        }
                    }
                }

                let area = self.world.get::<&Pos>(self.controlled).unwrap().get_area();
                if is_ship_launching(&self.world, area) {
                    view_buffer
                        .messages
                        .add("The ship leaves for the next planet.");
                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);

                    area::despawn_all_except_ship(&mut self.world, self.ship);
                    self.world.get::<&mut Ship>(self.ship).unwrap().status =
                        ShipStatus::NeedTwoCans;
                    for (_, health) in self.world.query_mut::<&mut Health>() {
                        health.restore_to_full();
                    }
                    self.load_location = true;
                } else {
                    view_buffer.capture_view(&self.world, self.controlled, &mut self.cache);
                }
            }
        }
    }
}

enum StopType {
    Win,
    Lose,
}

enum TickInterrupt {
    Lose,
    ControlCharacter(Entity),
}

fn tick(game: &mut Game, view_buffer: &mut view::Buffer) -> Result<(), TickInterrupt> {
    let world = &mut game.world;
    let controlled = game.controlled;

    decision_phase(world, controlled, view_buffer)?;

    ai::tick(world);

    action::tick(world, &mut game.rng, &mut view_buffer.messages, controlled);

    detect_low_health(world, &mut view_buffer.messages, controlled);

    handle_aftik_deaths(world, view_buffer, controlled);

    for (_, stamina) in world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }

    check_player_state(world, controlled)?;

    Ok(())
}

fn decision_phase(
    world: &mut World,
    controlled: Entity,
    view_buffer: &mut view::Buffer,
) -> Result<(), TickInterrupt> {
    if world.get::<&Action>(controlled).is_err() {
        mem::take(view_buffer).print();
        let (action, target) = parse_user_action(world, controlled)?;

        match target {
            Target::Controlled => world.insert_one(controlled, action).unwrap(),
            Target::Crew => {
                let area = world.get::<&Pos>(controlled).unwrap().get_area();
                insert_crew_action(world, area, action);
            }
        }
    }
    Ok(())
}

fn parse_user_action(world: &World, controlled: Entity) -> Result<(Action, Target), TickInterrupt> {
    loop {
        let input = read_input().to_lowercase();

        match command::try_parse_input(&input, world, controlled) {
            Ok(CommandResult::Action(action, target)) => return Ok((action, target)),
            Ok(CommandResult::ChangeControlled(new_aftik)) => {
                return Err(TickInterrupt::ControlCharacter(new_aftik));
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

fn check_player_state(world: &World, controlled: Entity) -> Result<(), TickInterrupt> {
    if world.get::<&CrewMember>(controlled).is_err() {
        if let Some((next_character, _)) = world.query::<()>().with::<&CrewMember>().iter().next() {
            Err(TickInterrupt::ControlCharacter(next_character))
        } else {
            Err(TickInterrupt::Lose)
        }
    } else {
        Ok(())
    }
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
