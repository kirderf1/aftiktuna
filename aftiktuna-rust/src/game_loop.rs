use crate::action::{combat, item, Action};
use crate::position::Pos;
use crate::status::Stamina;
use crate::view::{DisplayInfo, Messages};
use crate::{action, area, parse, status, view};
use hecs::{Entity, With, World};
use std::io::Write;
use std::{io, thread, time};

pub fn run() {
    let mut world = World::new();
    let mut messages = Messages::default();
    let mut cache = None;

    let (ship, aftik) = area::init(&mut world);

    println!(
        "You're playing as the aftik {}.",
        world.get::<DisplayInfo>(aftik).unwrap().name()
    );

    loop {
        for (_, stamina) in world.query_mut::<&mut Stamina>() {
            stamina.tick();
        }

        view::print(&world, aftik, &mut messages, &mut cache);

        if !status::is_alive(aftik, &world) {
            println!(
                "{} is dead.",
                DisplayInfo::find_definite_name(&world, aftik)
            );
            thread::sleep(time::Duration::from_secs(2));
            println!("You lost.");
            break;
        }

        if has_won(&world, ship, aftik) {
            println!("Congratulations, you won!");
            break;
        }

        decision_phase(&mut world, aftik);

        action_phase(&mut world, &mut messages, aftik);
    }
}

fn has_won(world: &World, ship: Entity, aftik: Entity) -> bool {
    if let Ok(pos) = world.get::<Pos>(aftik) {
        pos.is_in(ship) && item::is_holding::<item::FuelCan>(world)
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

        match parse::try_parse_input(&input, world, aftik) {
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
