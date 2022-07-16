use crate::action::{combat, item, Action, Aftik};
use crate::area::{Ship, ShipStatus};
use crate::position::Pos;
use crate::status::Stamina;
use crate::view::{DisplayInfo, Messages};
use crate::{action, area, command, status, view};
use hecs::{Entity, With, World};
use std::io::Write;
use std::{io, thread, time};

pub fn run() {
    let mut world = World::new();
    let mut messages = Messages::default();
    let mut cache = None;

    let mut aftik = area::init(&mut world);

    println!(
        "You're playing as the aftik {}.",
        DisplayInfo::find_name(&world, aftik)
    );

    loop {
        for (_, stamina) in world.query_mut::<&mut Stamina>() {
            stamina.tick();
        }

        view::print(&world, aftik, &mut messages, &mut cache);

        if has_won(&world, aftik) {
            println!("Congratulations, you won!");
            break;
        }

        decision_phase(&mut world, aftik);

        action_phase(&mut world, &mut messages, aftik);

        if !status::is_alive(aftik, &world) {
            view::print(&world, aftik, &mut messages, &mut None);
            thread::sleep(time::Duration::from_secs(2));
            println!(
                "{} is dead.",
                DisplayInfo::find_definite_name(&world, aftik)
            );
            item::drop_all_items(&mut world, aftik);
            world.despawn(aftik).unwrap();
        }

        if world.get::<Aftik>(aftik).is_err() {
            if let Some((next_aftik, _)) = world.query::<()>().with::<Aftik>().iter().next() {
                aftik = next_aftik;
                cache = None;
                println!(
                    "You're now playing as the aftik {}.",
                    DisplayInfo::find_name(&world, aftik)
                );
            } else {
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

fn decision_phase(world: &mut World, aftik: Entity) {
    let foes = world
        .query::<With<combat::IsFoe, ()>>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for foe in foes {
        action::foe_ai(world, foe);
    }

    if world.get::<Action>(aftik).is_err() {
        let action = parse_user_action(world, aftik);
        world.insert_one(aftik, action).unwrap();
    } else {
        thread::sleep(time::Duration::from_secs(2));
    }
}

fn parse_user_action(world: &World, aftik: Entity) -> Action {
    loop {
        let input = read_input().to_lowercase();

        match command::try_parse_input(&input, world, aftik) {
            Ok(Some(action)) => return action,
            Ok(None) => {}
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
