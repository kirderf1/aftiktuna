use crate::action::{item, Action, CrewMember};
use crate::area::{Locations, Ship, ShipStatus};
use crate::command::{CommandResult, Target};
use crate::position::Pos;
use crate::status::{Health, Stamina};
use crate::view::{DisplayInfo, Messages, StatusCache};
use crate::{action, ai, area, command, status, view};
use hecs::{Entity, World};
use rand::{thread_rng, Rng};
use std::{thread, time};

pub fn run() {
    let mut world = World::new();
    let mut messages = Messages::default();
    let mut rng = thread_rng();
    let mut cache = None;

    let mut locations = Locations::new(2);
    let (mut aftik, ship) = area::init(&mut world);

    println!(
        "You're playing as the aftik {}.",
        DisplayInfo::find_name(&world, aftik)
    );

    area::load_location(
        &mut world,
        &mut messages,
        ship,
        locations.pick_random(&mut rng).unwrap(),
    );

    loop {
        if let Err(stop_type) = tick(
            &mut world,
            &mut messages,
            &mut rng,
            &mut aftik,
            &mut cache,
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
    aftik: &mut Entity,
    cache: &mut Option<StatusCache>,
    locations: &mut Locations,
) -> Result<(), StopType> {
    for (_, stamina) in world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }

    view::print(world, *aftik, messages, cache);

    decision_phase(world, aftik, cache);

    ai::tick(world);

    action::tick(world, rng, messages, *aftik);

    handle_aftik_deaths(world, messages, *aftik);

    check_player_state(world, messages, aftik)?;

    check_ship_state(world, messages, rng, *aftik, cache, locations)?;

    Ok(())
}

fn decision_phase(world: &mut World, player: &mut Entity, cache: &mut Option<StatusCache>) {
    if world.get::<&Action>(*player).is_err() {
        let (action, target) = parse_user_action(world, player, cache);
        match target {
            Target::Controlled => world.insert_one(*player, action).unwrap(),
            Target::Crew => {
                let area = world.get::<&Pos>(*player).unwrap().get_area();
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
        .with::<&CrewMember>()
        .iter()
        .filter(|(_, pos)| pos.is_in(area))
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();
    for aftik in aftiks {
        world.insert_one(aftik, action.clone()).unwrap();
    }
}

fn parse_user_action(
    world: &World,
    aftik: &mut Entity,
    cache: &mut Option<StatusCache>,
) -> (Action, Target) {
    loop {
        let input = crate::read_input().to_lowercase();

        match command::try_parse_input(&input, world, *aftik) {
            Ok(CommandResult::Action(action, target)) => return (action, target),
            Ok(CommandResult::ChangeControlled(new_aftik)) => {
                *aftik = new_aftik;

                let message = format!(
                    "You're now playing as the aftik {}.",
                    DisplayInfo::find_definite_name(world, *aftik)
                );
                view::print(world, *aftik, &mut Messages::simple(message), cache);
            }
            Ok(CommandResult::None) => {}
            Err(message) => println!("{}", message),
        }
    }
}

fn handle_aftik_deaths(world: &mut World, messages: &mut Messages, controlled_aftik: Entity) {
    let dead_crew = world
        .query::<&Health>()
        .with::<&CrewMember>()
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
    aftik: &mut Entity,
) -> Result<(), StopType> {
    if world.get::<&CrewMember>(*aftik).is_err() {
        if let Some((next_aftik, _)) = world.query::<()>().with::<&CrewMember>().iter().next() {
            *aftik = next_aftik;
            messages.add(format!(
                "You're now playing as the aftik {}.",
                DisplayInfo::find_name(world, *aftik)
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
    aftik: Entity,
    cache: &mut Option<StatusCache>,
    locations: &mut Locations,
) -> Result<(), StopType> {
    let area = world.get::<&Pos>(aftik).unwrap().get_area();
    if is_ship_launching(world, area) {
        let ship = area;
        messages.add("The ship leaves for the next planet.".to_string());
        view::print(world, aftik, messages, cache);

        if let Some(location_name) = locations.pick_random(rng) {
            area::despawn_all_except_ship(world, ship);
            world.get::<&mut Ship>(ship).unwrap().status = ShipStatus::NeedTwoCans;
            for (_, health) in world.query_mut::<&mut Health>() {
                health.restore_to_full();
            }

            area::load_location(world, messages, ship, location_name);
        } else {
            return Err(StopType::Win);
        }
    }
    Ok(())
}

fn is_ship_launching(world: &World, area: Entity) -> bool {
    world
        .get::<&Ship>(area)
        .map(|ship| ship.status == ShipStatus::Launching)
        .unwrap_or(false)
}
