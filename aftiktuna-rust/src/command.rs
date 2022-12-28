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
    let parse = Parse::new(input);
    None.or_else(|| {
        parse
            .literal("take")
            .map(|parse| take(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("wield")
            .map(|parse| wield(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("enter")
            .map(|parse| enter(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("force")
            .map(|parse| force(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("attack")
            .map(|parse| attack(&parse, world, aftik))
    })
    .or_else(|| parse.literal("wait").map(|parse| wait(&parse)))
    .or_else(|| {
        parse
            .literal("rest")
            .map(|parse| rest(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("launch")
            .map(|parse| launch(&parse, world, aftik))
    })
    .or_else(|| parse.literal("status").map(|parse| status(&parse, world)))
    .or_else(|| {
        parse
            .literal("control")
            .map(|parse| control(&parse, world, aftik))
    })
    .unwrap_or_else(|| Err(format!("Unexpected input: \"{}\"", input)))
}

fn take(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    None.or_else(|| {
        parse
            .literal("all")
            .and_then(|parse| parse.done(|| action_result(Action::TakeAll)))
    })
    .unwrap_or_else(|| {
        parse.take_remaining(|name| {
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
    })
}

fn wield(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
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

fn enter(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
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

fn force(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
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

fn attack(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse
        .done(|| action_result(Action::AttackNearest(combat::Target::Foe)))
        .unwrap_or_else(|| {
            parse.take_remaining(|name| {
                let area = world.get::<Pos>(aftik).unwrap().get_area();
                world
                    .query::<With<combat::IsFoe, (&Pos, &DisplayInfo)>>()
                    .iter()
                    .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
                    .map(|(target, _)| action_result(Action::Attack(target)))
                    .unwrap_or_else(|| Err("There is no such target here.".to_string()))
            })
        })
}

fn wait(parse: &Parse) -> Result<CommandResult, String> {
    parse.done_or_err(|| action_result(Action::Wait))
}

fn rest(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
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

fn launch(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
    parse
        .literal("ship")
        .map(|parse| launch_ship(&parse, world, aftik))
        .unwrap_or_else(|| Err(format!("Unexpected argument after \"launch\"")))
}

fn launch_ship(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
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

fn status(parse: &Parse, world: &World) -> Result<CommandResult, String> {
    parse.done_or_err(|| {
        println!("Crew:");
        for (aftik, _) in world.query::<()>().with::<Aftik>().iter() {
            view::print_full_status(world, aftik);
        }
        Ok(CommandResult::None)
    })
}

fn control(parse: &Parse, world: &World, aftik: Entity) -> Result<CommandResult, String> {
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
