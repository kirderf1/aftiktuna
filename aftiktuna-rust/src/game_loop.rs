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
use std::{io, thread, time};

struct Game {
    world: World,
    rng: ThreadRng,
    controlled: Entity,
    ship: Entity,
    load_location: bool,
}

pub fn run(mut locations: Locations) {
    let mut world = World::new();
    let rng = thread_rng();
    let mut cache = StatusCache::default();

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
    };

    loop {
        let mut view_buffer = view::Buffer::default();
        if game.load_location {
            game.load_location = false;
            game.world.get::<&mut Ship>(game.ship).unwrap().status = ShipStatus::NeedTwoCans;
            for (_, health) in game.world.query_mut::<&mut Health>() {
                health.restore_to_full();
            }
            let location = match locations.next(&mut game.rng) {
                PickResult::None => {
                    println!();
                    println!("Congratulations, you won!");
                    return;
                }
                PickResult::Location(location) => location,
                PickResult::Choice(choice) => loop {
                    if let Some(location) =
                        locations.try_make_choice(&choice, read_input(), &mut game.rng)
                    {
                        break location;
                    }
                },
            };
            area::load_location(
                &mut game.world,
                &mut view_buffer.messages,
                game.ship,
                &location,
            );
        } else if let Err(stop_type) = tick(&mut game, &mut view_buffer) {
            match stop_type {
                StopType::Lose => {
                    view_buffer.print();
                    println!();
                    println!("You lost.");
                    return;
                }
                StopType::ControlCharacter(character) => {
                    game.controlled = character;

                    view_buffer.messages.add(format!(
                        "You're now playing as the aftik {}.",
                        NameData::find(&game.world, game.controlled).definite()
                    ));
                }
            }
        }

        let area = game.world.get::<&Pos>(game.controlled).unwrap().get_area();
        if is_ship_launching(&game.world, area) {
            view_buffer
                .messages
                .add("The ship leaves for the next planet.");

            area::despawn_all_except_ship(&mut game.world, game.ship);
            game.load_location = true;
        }

        view_buffer.capture_view(&game.world, game.controlled, &mut cache);
        view_buffer.print();
    }
}

enum StopType {
    Lose,
    ControlCharacter(Entity),
}

fn tick(game: &mut Game, view_buffer: &mut view::Buffer) -> Result<(), StopType> {
    let world = &mut game.world;
    let controlled = game.controlled;

    decision_phase(world, controlled)?;

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

fn decision_phase(world: &mut World, controlled: Entity) -> Result<(), StopType> {
    if world.get::<&Action>(controlled).is_err() {
        let (action, target) = parse_user_action(world, controlled)?;
        match target {
            Target::Controlled => world.insert_one(controlled, action).unwrap(),
            Target::Crew => {
                let area = world.get::<&Pos>(controlled).unwrap().get_area();
                insert_crew_action(world, area, action);
            }
        }
    } else {
        thread::sleep(time::Duration::from_secs(2));
    }
    Ok(())
}

fn parse_user_action(world: &World, controlled: Entity) -> Result<(Action, Target), StopType> {
    loop {
        let input = read_input().to_lowercase();

        match command::try_parse_input(&input, world, controlled) {
            Ok(CommandResult::Action(action, target)) => return Ok((action, target)),
            Ok(CommandResult::ChangeControlled(new_aftik)) => {
                return Err(StopType::ControlCharacter(new_aftik));
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

fn check_player_state(world: &World, controlled: Entity) -> Result<(), StopType> {
    if world.get::<&CrewMember>(controlled).is_err() {
        if let Some((next_character, _)) = world.query::<()>().with::<&CrewMember>().iter().next() {
            Err(StopType::ControlCharacter(next_character))
        } else {
            Err(StopType::Lose)
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
