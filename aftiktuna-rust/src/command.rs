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
        .literal("take", |parse| take(parse, world, aftik))
        .literal("wield", |parse| wield(parse, world, aftik))
        .literal("enter", |parse| enter(parse, world, aftik))
        .literal("force", |parse| force(parse, world, aftik))
        .literal("attack", |parse| attack(parse, world, aftik))
        .literal("wait", wait)
        .literal("rest", |parse| rest(parse, world, aftik))
        .literal("launch", |parse| launch(parse, world, aftik))
        .literal("status", |parse| status(parse, world))
        .literal("control", |parse| control(parse, world, aftik))
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}

fn take(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse
        .literal("all", |parse| {
            parse.done_or_err(|| action_result(Action::TakeAll))
        })
        .or_else_remaining(|name| {
            let aftik_pos = *world.get::<Pos>(aftik).unwrap();
            world
                .query::<With<item::Item, (&Pos, &DisplayInfo)>>()
                .iter()
                .filter(|(_, (pos, display_info))| {
                    pos.is_in(aftik_pos.get_area()) && display_info.matches(name)
                })
                .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
                .map(|(item, (_, display_info))| {
                    action_result(Action::TakeItem(
                        item,
                        display_info.definite_name().to_string(),
                    ))
                })
                .unwrap_or_else(|| Err(format!("There is no {} here to pick up.", name)))
        })
}

fn wield(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse.take_remaining(|name| {
        None.or_else(|| {
            world
                .query::<(&DisplayInfo, &item::InInventory)>()
                .with::<item::CanWield>()
                .with::<item::Item>()
                .iter()
                .find(|(_, (display_info, in_inventory))| {
                    display_info.matches(name) && in_inventory.held_by(aftik)
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
                    pos.is_in(aftik_pos.get_area()) && display_info.matches(name)
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
                name,
                DisplayInfo::find_definite_name(world, aftik)
            ))
        })
    })
}

fn enter(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse.take_remaining(|name| {
        let area = world.get::<Pos>(aftik).unwrap().get_area();
        world
            .query::<With<door::Door, (&Pos, &DisplayInfo)>>()
            .iter()
            .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
            .map(|(door, _)| crew_action(Action::EnterDoor(door)))
            .unwrap_or_else(|| Err("There is no such door here to go through.".to_string()))
    })
}

fn force(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse.take_remaining(|name| {
        let area = world.get::<Pos>(aftik).unwrap().get_area();
        world
            .query::<With<door::Door, (&Pos, &DisplayInfo)>>()
            .iter()
            .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
            .map(|(door, _)| action_result(Action::ForceDoor(door)))
            .unwrap_or_else(|| Err("There is no such door here.".to_string()))
    })
}

fn attack(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse
        .done(|| action_result(Action::AttackNearest(combat::Target::Foe)))
        .or_else_remaining(|name| {
            let area = world.get::<Pos>(aftik).unwrap().get_area();
            world
                .query::<With<combat::IsFoe, (&Pos, &DisplayInfo)>>()
                .iter()
                .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
                .map(|(target, _)| action_result(Action::Attack(target)))
                .unwrap_or_else(|| Err("There is no such target here.".to_string()))
        })
}

fn wait(parse: Parse) -> Result<CommandResult, String> {
    parse.done_or_err(|| action_result(Action::Wait))
}

fn rest(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse.done_or_err(|| {
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
    })
}

fn launch(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse
        .literal("ship", |parse| launch_ship(parse, world, aftik))
        .or_else_err(|| "Unexpected argument after \"launch\"".to_string())
}

fn launch_ship(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse.done_or_err(|| {
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
    })
}

fn status(parse: Parse, world: &World) -> Result<CommandResult, String> {
    parse.done_or_err(|| {
        println!("Crew:");
        for (aftik, _) in world.query::<()>().with::<Aftik>().iter() {
            view::print_full_status(world, aftik);
        }
        Ok(CommandResult::None)
    })
}

fn control(parse: Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse.take_remaining(|name| {
        let (new_aftik, _) = world
            .query::<&DisplayInfo>()
            .with::<Aftik>()
            .iter()
            .find(|(_, display_info)| display_info.matches(name))
            .ok_or_else(|| "There is no crew member by that name.".to_string())?;

        if new_aftik == aftik {
            Err("You're already in control of them.".to_string())
        } else {
            Ok(CommandResult::ChangeControlled(new_aftik))
        }
    })
}
