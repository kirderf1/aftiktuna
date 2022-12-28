use crate::action::item::FuelCan;
use crate::action::{combat, door, item, Action, Aftik};
use crate::area::Ship;
use crate::game_loop::Target;
use crate::position::Pos;
use crate::view::DisplayInfo;
use crate::{status, view};
use hecs::{Entity, With, World};
use parse::Parse;

mod parse;

pub enum CommandResult {
    Action(Action, Target),
    ChangeControlled(Entity),
    None,
}

fn action_result(action: Action) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action, Target::Controlled))
}

fn crew_action(action: Action) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action, Target::Crew))
}

pub fn try_parse_input(input: &str, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    Parse::new(input)
        .literal("take", |parse| {
            parse
                .literal("all", |parse| {
                    parse.done_or_err(|| action_result(Action::TakeAll))
                })
                .or_else_remaining(|item_name| take(item_name, world, aftik))
        })
        .literal("give", |parse| {
            parse.match_against(
                aftik_targets(world),
                |parse, receiver| {
                    parse.take_remaining(|item_name| give(receiver, item_name, world, aftik))
                },
                |input| Err(format!("\"{}\" not a valid target", input)),
            )
        })
        .literal("wield", |parse| {
            parse.take_remaining(|item_name| wield(item_name, world, aftik))
        })
        .literal("enter", |parse| {
            parse.take_remaining(|door_name| enter(door_name, world, aftik))
        })
        .literal("force", |parse| {
            parse.take_remaining(|door_name| force(door_name, world, aftik))
        })
        .literal("attack", |parse| {
            parse
                .done(|| attack_any(world, aftik))
                .or_else_remaining(|target_name| attack(target_name, world, aftik))
        })
        .literal("wait", |parse| {
            parse.done_or_err(|| action_result(Action::Wait))
        })
        .literal("rest", |parse| parse.done_or_err(|| rest(world, aftik)))
        .literal("launch", |parse| {
            parse
                .literal("ship", |parse| {
                    parse.done_or_err(|| launch_ship(world, aftik))
                })
                .or_else_err(|| "Unexpected argument after \"launch\"".to_string())
        })
        .literal("status", |parse| parse.done_or_err(|| status(world)))
        .literal("control", |parse| {
            parse.take_remaining(|aftik_name| control(world, aftik, aftik_name))
        })
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}

fn aftik_targets(world: &World) -> Vec<(String, Entity)> {
    world
        .query::<&DisplayInfo>()
        .with::<Aftik>()
        .iter()
        .map(|(entity, display_info)| (display_info.name().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

fn take(item_name: &str, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let aftik_pos = *world.get::<Pos>(aftik).unwrap();
    world
        .query::<With<item::Item, (&Pos, &DisplayInfo)>>()
        .iter()
        .filter(|(_, (pos, display_info))| {
            pos.is_in(aftik_pos.get_area()) && display_info.matches(item_name)
        })
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, display_info))| {
            action_result(Action::TakeItem(
                item,
                display_info.definite_name().to_string(),
            ))
        })
        .unwrap_or_else(|| Err(format!("There is no {} here to pick up.", item_name)))
}

fn give(
    receiver: Entity,
    item_name: &str,
    world: &World,
    aftik: Entity,
) -> Result<CommandResult, String> {
    if aftik == receiver {
        return Err(format!("{} can't give an item to themselves.",
            DisplayInfo::find_definite_name(world, aftik)));
    }

    world
        .query::<(&DisplayInfo, &item::InInventory)>()
        .with::<item::Item>()
        .iter()
        .find(|(_, (display_info, in_inventory))| {
            display_info.matches(item_name) && in_inventory.held_by(aftik)
        })
        .map(|(item, _)| action_result(Action::GiveItem(item, receiver)))
        .unwrap_or_else(|| {
            Err(format!(
                "{} has no {} to give.",
                DisplayInfo::find_definite_name(world, aftik),
                item_name,
            ))
        })
}

fn wield(item_name: &str, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    None.or_else(|| {
        world
            .query::<(&DisplayInfo, &item::InInventory)>()
            .with::<item::CanWield>()
            .with::<item::Item>()
            .iter()
            .find(|(_, (display_info, in_inventory))| {
                display_info.matches(item_name) && in_inventory.held_by(aftik)
            })
            .map(|(item, (display_info, _))| {
                action_result(Action::Wield(
                    item,
                    display_info.definite_name().to_string(),
                ))
            })
    })
    .or_else(|| {
        let aftik_pos = *world.get::<Pos>(aftik).unwrap();
        world
            .query::<(&Pos, &DisplayInfo)>()
            .with::<item::CanWield>()
            .with::<item::Item>()
            .iter()
            .filter(|(_, (pos, display_info))| {
                pos.is_in(aftik_pos.get_area()) && display_info.matches(item_name)
            })
            .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
            .map(|(item, (_, display_info))| {
                action_result(Action::Wield(
                    item,
                    display_info.definite_name().to_string(),
                ))
            })
    })
    .unwrap_or_else(|| {
        Err(format!(
            "There is no {} that {} can wield.",
            item_name,
            DisplayInfo::find_definite_name(world, aftik)
        ))
    })
}

fn enter(door_name: &str, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    world
        .query::<With<door::Door, (&Pos, &DisplayInfo)>>()
        .iter()
        .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(door_name))
        .map(|(door, _)| crew_action(Action::EnterDoor(door)))
        .unwrap_or_else(|| Err("There is no such door here to go through.".to_string()))
}

fn force(door_name: &str, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    world
        .query::<With<door::Door, (&Pos, &DisplayInfo)>>()
        .iter()
        .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(door_name))
        .map(|(door, _)| action_result(Action::ForceDoor(door)))
        .unwrap_or_else(|| Err("There is no such door here.".to_string()))
}

fn attack_any(world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    if world.query::<&Pos>()
        .with::<combat::IsFoe>()
        .iter()
        .any(|(_, pos)| pos.is_in(area)) {
        action_result(Action::AttackNearest(combat::Target::Foe))
    } else {
        Err("There is no appropriate target to attack here.".to_string())
    }
}

fn attack(target_name: &str, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    world
        .query::<With<combat::IsFoe, (&Pos, &DisplayInfo)>>()
        .iter()
        .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(target_name))
        .map(|(target, _)| action_result(Action::Attack(target)))
        .unwrap_or_else(|| Err("There is no such target here.".to_string()))
}

fn rest(world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    if world
        .query::<With<combat::IsFoe, &Pos>>()
        .iter()
        .any(|(_, pos)| pos.is_in(area))
    {
        Err("This area is not safe to rest in.".to_string())
    } else {
        let need_rest = world
            .get::<status::Stamina>(aftik)
            .map(|stamina| stamina.need_rest())
            .unwrap_or(false);

        if need_rest {
            action_result(Action::Rest(true))
        } else {
            Err(format!(
                "{} is already rested.",
                DisplayInfo::find_definite_name(world, aftik)
            ))
        }
    }
}

fn launch_ship(world: &World, aftik: Entity) -> Result<CommandResult, String> {
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    if !item::is_holding::<FuelCan>(world, aftik) {
        return Err(format!(
            "{} needs a fuel can to launch the ship.",
            DisplayInfo::find_definite_name(world, aftik)
        ));
    }
    world.get::<Ship>(area).map_err(|_| {
        format!(
            "{} needs to be near the ship in order to launch it.",
            DisplayInfo::find_definite_name(world, aftik)
        )
    })?;
    action_result(Action::Launch)
}

fn status(world: &World) -> Result<CommandResult, String> {
    println!("Crew:");
    for (aftik, _) in world.query::<()>().with::<Aftik>().iter() {
        view::print_full_status(world, aftik);
    }
    Ok(CommandResult::None)
}

fn control(world: &World, aftik: Entity, aftik_name: &str) -> Result<CommandResult, String> {
    let (new_aftik, _) = world
        .query::<&DisplayInfo>()
        .with::<Aftik>()
        .iter()
        .find(|(_, display_info)| display_info.matches(aftik_name))
        .ok_or_else(|| "There is no crew member by that name.".to_string())?;

    if new_aftik == aftik {
        Err("You're already in control of them.".to_string())
    } else {
        Ok(CommandResult::ChangeControlled(new_aftik))
    }
}
