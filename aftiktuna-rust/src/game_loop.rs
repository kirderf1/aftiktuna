use crate::action::{combat, item, Action, Aftik};
use crate::area::{Ship, ShipStatus};
use crate::command::CommandResult;
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{DisplayInfo, Messages};
use crate::{action, area, command, status, view};
use hecs::{Entity, With, World};
use std::io::Write;
use std::{io, thread, time};

struct PlayerControlled {
    entity: Entity,
    cache: Option<view::StatusCache>,
}

impl PlayerControlled {
    fn new(entity: Entity) -> PlayerControlled {
        PlayerControlled {
            entity,
            cache: None,
        }
    }
}

pub fn run() {
    let mut world = World::new();
    let mut messages = Messages::default();

    let aftik = area::init(&mut world);
    let mut aftik = PlayerControlled::new(aftik);

    println!(
        "You're playing as the aftik {}.",
        DisplayInfo::find_name(&world, aftik.entity)
    );

    loop {
        for (_, stamina) in world.query_mut::<&mut Stamina>() {
            stamina.tick();
        }

        view::print(&world, aftik.entity, &mut messages, &mut aftik.cache);

        if has_won(&world, aftik.entity) {
            println!("Congratulations, you won!");
            break;
        }

        decision_phase(&mut world, &mut aftik);

        action_phase(&mut world, &mut messages, aftik.entity);

        handle_aftik_deaths(&mut world, &mut messages, aftik.entity);

        if world.get::<Aftik>(aftik.entity).is_err() {
            if let Some((next_aftik, _)) = world.query::<()>().with::<Aftik>().iter().next() {
                aftik = PlayerControlled::new(next_aftik);
                messages.add(format!(
                    "You're now playing as the aftik {}.",
                    DisplayInfo::find_name(&world, aftik.entity)
                ));
            } else {
                messages.print_and_clear();
                println!("You lost.");
                break;
            }
        }
    }
}

fn has_won(world: &World, aftik: Entity) -> bool {
    if let Ok(pos) = world.get::<Pos>(aftik) {
        world
            .get::<Ship>(pos.get_area())
            .map(|ship| ship.0 == ShipStatus::Launching)
            .unwrap_or(false)
    } else {
        false
    }
}

fn decision_phase(world: &mut World, aftik: &mut PlayerControlled) {
    let foes = world
        .query::<With<combat::IsFoe, ()>>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for foe in foes {
        action::foe_ai(world, foe);
    }

    if world.get::<Action>(aftik.entity).is_err() {
        let action = parse_user_action(world, aftik);
        world.insert_one(aftik.entity, action).unwrap();
    } else {
        thread::sleep(time::Duration::from_secs(2));
    }
}

fn parse_user_action(world: &World, aftik: &mut PlayerControlled) -> Action {
    loop {
        let input = read_input().to_lowercase();

        match command::try_parse_input(&input, world, aftik.entity) {
            Ok(CommandResult::Action(action)) => return action,
            Ok(CommandResult::ChangeControlled(new_aftik)) => {
                *aftik = PlayerControlled::new(new_aftik);

                let message = format!(
                    "You're now playing as the aftik {}.",
                    DisplayInfo::find_definite_name(world, aftik.entity)
                );
                view::print(
                    world,
                    aftik.entity,
                    &mut Messages::simple(message),
                    &mut aftik.cache,
                );
            }
            Ok(CommandResult::None) => {}
            Err(message) => println!("{}", message),
        }
    }
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

fn action_phase(world: &mut World, messages: &mut Messages, aftik: Entity) {
    let mut entities = world
        .query::<With<Action, &status::Stats>>()
        .iter()
        .map(|(entity, stats)| (entity, stats.agility))
        .collect::<Vec<_>>();
    entities.sort_by(|(_, agility1), (_, agility2)| agility2.cmp(agility1));
    let entities = entities
        .iter()
        .map(|(entity, _)| *entity)
        .collect::<Vec<_>>();

    for entity in entities {
        if !status::is_alive(entity, world) {
            continue;
        }

        if let Ok(action) = world.remove_one::<Action>(entity) {
            action::perform(world, entity, action, aftik, messages);
        }
    }
}

fn handle_aftik_deaths(world: &mut World, messages: &mut Messages, controlled_aftik: Entity) {
    if !status::is_alive(controlled_aftik, world) {
        view::print(world, controlled_aftik, messages, &mut None);
        thread::sleep(time::Duration::from_secs(2));
    }
    let dead_crew = world
        .query::<&Health>()
        .with::<Aftik>()
        .iter()
        .filter(|(_, health)| health.is_dead())
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();
    for aftik in dead_crew {
        messages.add(format!(
            "{} is dead.",
            DisplayInfo::find_definite_name(world, aftik)
        ));
        item::drop_all_items(world, aftik);
        world.despawn(aftik).unwrap();
    }
}
