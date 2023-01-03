use crate::action::{item, Action, Aftik};
use crate::area::{Locations, Ship, ShipStatus};
use crate::command::{CommandResult, Target};
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{DisplayInfo, Messages};
use crate::{action, ai, area, command, status, view};
use hecs::{Entity, World};
use rand::{thread_rng, Rng};
use std::{thread, time};

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
    let mut rng = thread_rng();

    let mut locations = Locations::new(2);
    let (aftik, ship_exit) = area::init(&mut world);
    area::load_location(
        &mut world,
        ship_exit,
        locations.pick_random(&mut rng).unwrap(),
    );
    let mut aftik = PlayerControlled::new(aftik);

    println!(
        "You're playing as the aftik {}.",
        DisplayInfo::find_name(&world, aftik.entity)
    );

    loop {
        if let Err(stop_type) = tick(
            &mut world,
            &mut messages,
            &mut rng,
            ship_exit,
            &mut aftik,
            &mut locations,
        ) {
            match stop_type {
                StopType::Lose => {
                    println!();
                    println!("You lost.");
                }
                StopType::Win => {
                    println!();
                    println!("Congratulations, you won!");
                }
            }
            break;
        }
    }
}

enum StopType {
    Lose,
    Win,
}

fn tick(
    world: &mut World,
    messages: &mut Messages,
    rng: &mut impl Rng,
    ship_exit: Pos,
    aftik: &mut PlayerControlled,
    locations: &mut Locations,
) -> Result<(), StopType> {
    for (_, stamina) in world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }

    view::print(world, aftik.entity, messages, &mut aftik.cache);

    decision_phase(world, aftik);

    ai::tick(world);

    action::tick(world, rng, messages, aftik.entity);

    handle_aftik_deaths(world, messages, aftik.entity);

    check_player_state(world, messages, aftik)?;

    check_ship_state(world, messages, rng, ship_exit, aftik, locations)?;

    Ok(())
}

fn decision_phase(world: &mut World, player: &mut PlayerControlled) {
    if world.get::<&Action>(player.entity).is_err() {
        let (action, target) = parse_user_action(world, player);
        match target {
            Target::Controlled => world.insert_one(player.entity, action).unwrap(),
            Target::Crew => {
                let area = world.get::<&Pos>(player.entity).unwrap().get_area();
                insert_crew_action(world, area, action);
            }
        }
    } else {
        thread::sleep(time::Duration::from_secs(2));
    }
}

fn insert_crew_action(world: &mut World, area: Entity, action: Action) {
    let aftiks = world
        .query::<&Pos>()
        .with::<&Aftik>()
        .iter()
        .filter(|(_, pos)| pos.is_in(area))
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();
    for aftik in aftiks {
        world.insert_one(aftik, action.clone()).unwrap();
    }
}

fn parse_user_action(world: &World, aftik: &mut PlayerControlled) -> (Action, Target) {
    loop {
        let input = crate::read_input().to_lowercase();

        match command::try_parse_input(&input, world, aftik.entity) {
            Ok(CommandResult::Action(action, target)) => return (action, target),
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

fn handle_aftik_deaths(world: &mut World, messages: &mut Messages, controlled_aftik: Entity) {
    let dead_crew = world
        .query::<&Health>()
        .with::<&Aftik>()
        .iter()
        .filter(|(_, health)| health.is_dead())
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();

    for &aftik in &dead_crew {
        messages.add(format!(
            "{} is dead.",
            DisplayInfo::find_definite_name(world, aftik)
        ));
    }

    if !status::is_alive(controlled_aftik, world) {
        view::print(world, controlled_aftik, messages, &mut None);
        thread::sleep(time::Duration::from_secs(2));
    }

    for aftik in dead_crew {
        item::drop_all_items(world, aftik);
        world.despawn(aftik).unwrap();
    }
}

fn check_player_state(
    world: &World,
    messages: &mut Messages,
    aftik: &mut PlayerControlled,
) -> Result<(), StopType> {
    if world.get::<&Aftik>(aftik.entity).is_err() {
        if let Some((next_aftik, _)) = world.query::<()>().with::<&Aftik>().iter().next() {
            *aftik = PlayerControlled::new(next_aftik);
            messages.add(format!(
                "You're now playing as the aftik {}.",
                DisplayInfo::find_name(world, aftik.entity)
            ));
        } else {
            return Err(StopType::Lose);
        }
    }
    Ok(())
}

fn check_ship_state(
    world: &mut World,
    messages: &mut Messages,
    rng: &mut impl Rng,
    ship_exit: Pos,
    aftik: &mut PlayerControlled,
    locations: &mut Locations,
) -> Result<(), StopType> {
    if is_ship_launching(world, aftik.entity) {
        messages.add("The ship leaves for the next planet.".to_string());
        view::print(world, aftik.entity, messages, &mut aftik.cache);

        if let Some(location_name) = locations.pick_random(rng) {
            area::despawn_all_except_ship(world, ship_exit.get_area());
            world
                .insert_one(ship_exit.get_area(), Ship(ShipStatus::NeedTwoCans))
                .unwrap();
            area::load_location(world, ship_exit, location_name);
        } else {
            return Err(StopType::Win);
        }
    }
    Ok(())
}

fn is_ship_launching(world: &World, aftik: Entity) -> bool {
    if let Ok(pos) = world.get::<&Pos>(aftik) {
        world
            .get::<&Ship>(pos.get_area())
            .map(|ship| ship.0 == ShipStatus::Launching)
            .unwrap_or(false)
    } else {
        false
    }
}
