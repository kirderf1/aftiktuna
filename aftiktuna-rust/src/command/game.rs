use crate::action::item::{is_holding, Held};
use crate::action::trade::Shopkeeper;
use crate::action::{combat, door, Action, CrewMember, Recruitable};
use crate::area::Ship;
use crate::command::parse::Parse;
use crate::command::CommandResult;
use crate::item::FuelCan;
use crate::position::Pos;
use crate::view::{DisplayInfo, NameData};
use crate::{command, item, status};
use hecs::{Entity, World};

pub fn parse(input: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    Parse::new(input)
        .literal("take", |parse| {
            parse
                .literal("all", |parse| {
                    parse.done_or_err(|| command::action_result(Action::TakeAll))
                })
                .or_else_remaining(|item_name| take(item_name, world, character))
        })
        .literal("give", |parse| {
            parse.match_against(
                crew_targets(world),
                |parse, receiver| {
                    parse.take_remaining(|item_name| give(receiver, item_name, world, character))
                },
                |input| Err(format!("\"{}\" not a valid target", input)),
            )
        })
        .literal("wield", |parse| {
            parse.take_remaining(|item_name| wield(item_name, world, character))
        })
        .literal("enter", |parse| {
            parse.take_remaining(|door_name| enter(door_name, world, character))
        })
        .literal("force", |parse| {
            parse.take_remaining(|door_name| force(door_name, world, character))
        })
        .literal("attack", |parse| {
            parse
                .done(|| attack_any(world, character))
                .or_else_remaining(|target_name| attack(target_name, world, character))
        })
        .literal("wait", |parse| {
            parse.done_or_err(|| command::action_result(Action::Wait))
        })
        .literal("rest", |parse| parse.done_or_err(|| rest(world, character)))
        .literal("launch", |parse| {
            parse
                .literal("ship", |parse| {
                    parse.done_or_err(|| launch_ship(world, character))
                })
                .or_else_err(|| "Unexpected argument after \"launch\"".to_string())
        })
        .literal("status", |parse| {
            parse.done_or_err(|| command::status(world, character))
        })
        .literal("control", |parse| {
            parse.take_remaining(|target_name| control(world, character, target_name))
        })
        .literal("trade", |parse| {
            parse.done_or_err(|| trade(world, character))
        })
        .literal("recruit", |parse| {
            parse.match_against(
                recruit_targets(world, character),
                |parse, target| {
                    parse.done_or_err(|| command::action_result(Action::Recruit(target)))
                },
                |input| Err(format!("\"{}\" not a valid recruitment target", input)),
            )
        })
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}

fn crew_targets(world: &World) -> Vec<(String, Entity)> {
    world
        .query::<&DisplayInfo>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, display_info)| (display_info.name().base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

fn take(item_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(&Pos, &DisplayInfo)>()
        .with::<&item::Item>()
        .iter()
        .filter(|(_, (pos, display_info))| {
            pos.is_in(character_pos.get_area()) && display_info.name().matches(item_name)
        })
        .min_by_key(|(_, (pos, _))| pos.distance_to(character_pos))
        .map(|(item, (_, display_info))| {
            command::action_result(Action::TakeItem(item, display_info.name().definite()))
        })
        .unwrap_or_else(|| Err(format!("There is no {} here to pick up.", item_name)))
}

fn give(
    receiver: Entity,
    item_name: &str,
    world: &World,
    character: Entity,
) -> Result<CommandResult, String> {
    if character == receiver {
        return Err(format!(
            "{} can't give an item to themselves.",
            NameData::find(world, character).definite()
        ));
    }

    world
        .query::<(&DisplayInfo, &Held)>()
        .with::<&item::Item>()
        .iter()
        .find(|(_, (display_info, held))| {
            display_info.name().matches(item_name) && held.held_by(character)
        })
        .map(|(item, _)| command::action_result(Action::GiveItem(item, receiver)))
        .unwrap_or_else(|| {
            Err(format!(
                "{} has no {} to give.",
                NameData::find(world, character).definite(),
                item_name,
            ))
        })
}

fn wield(item_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    None.or_else(|| {
        world
            .query::<(&DisplayInfo, &Held)>()
            .with::<&item::CanWield>()
            .with::<&item::Item>()
            .iter()
            .find(|(_, (display_info, held))| {
                display_info.name().matches(item_name) && held.is_in_inventory(character)
            })
            .map(|(item, (display_info, _))| {
                command::action_result(Action::Wield(item, display_info.name().clone()))
            })
    })
    .or_else(|| {
        let character_pos = *world.get::<&Pos>(character).unwrap();
        world
            .query::<(&Pos, &DisplayInfo)>()
            .with::<&item::CanWield>()
            .with::<&item::Item>()
            .iter()
            .filter(|(_, (pos, display_info))| {
                pos.is_in(character_pos.get_area()) && display_info.name().matches(item_name)
            })
            .min_by_key(|(_, (pos, _))| pos.distance_to(character_pos))
            .map(|(item, (_, display_info))| {
                command::action_result(Action::Wield(item, display_info.name().clone()))
            })
    })
    .unwrap_or_else(|| {
        Err(format!(
            "There is no {} that {} can wield.",
            item_name,
            NameData::find(world, character).definite()
        ))
    })
}

fn enter(door_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    world
        .query::<(&Pos, &DisplayInfo)>()
        .with::<&door::Door>()
        .iter()
        .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.name().matches(door_name))
        .map(|(door, _)| command::crew_action(Action::EnterDoor(door)))
        .unwrap_or_else(|| Err("There is no such door here to go through.".to_string()))
}

fn force(door_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    world
        .query::<(&Pos, &DisplayInfo)>()
        .with::<&door::Door>()
        .iter()
        .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.name().matches(door_name))
        .map(|(door, _)| command::action_result(Action::ForceDoor(door)))
        .unwrap_or_else(|| Err("There is no such door here.".to_string()))
}

fn attack_any(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    if world
        .query::<&Pos>()
        .with::<&combat::IsFoe>()
        .iter()
        .any(|(_, pos)| pos.is_in(area))
    {
        command::action_result(Action::AttackNearest(combat::Target::Foe))
    } else {
        Err("There is no appropriate target to attack here.".to_string())
    }
}

fn attack(target_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    world
        .query::<(&Pos, &DisplayInfo)>()
        .with::<&combat::IsFoe>()
        .iter()
        .find(|(_, (pos, display_info))| {
            pos.is_in(area) && display_info.name().matches(target_name)
        })
        .map(|(target, _)| command::action_result(Action::Attack(target)))
        .unwrap_or_else(|| Err("There is no such target here.".to_string()))
}

fn rest(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    if world
        .query::<&Pos>()
        .with::<&combat::IsFoe>()
        .iter()
        .any(|(_, pos)| pos.is_in(area))
    {
        Err("This area is not safe to rest in.".to_string())
    } else {
        let need_rest = world
            .get::<&status::Stamina>(character)
            .map(|stamina| stamina.need_rest())
            .unwrap_or(false);

        if need_rest {
            command::action_result(Action::Rest(true))
        } else {
            Err(format!(
                "{} is already rested.",
                NameData::find(world, character).definite()
            ))
        }
    }
}

fn launch_ship(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    if !is_holding::<FuelCan>(world, character) {
        return Err(format!(
            "{} needs a fuel can to launch the ship.",
            NameData::find(world, character).definite()
        ));
    }
    world.get::<&Ship>(area).map_err(|_| {
        format!(
            "{} needs to be near the ship in order to launch it.",
            NameData::find(world, character).definite()
        )
    })?;
    command::action_result(Action::Launch)
}

fn control(world: &World, character: Entity, target_name: &str) -> Result<CommandResult, String> {
    let (new_character, _) = world
        .query::<&DisplayInfo>()
        .with::<&CrewMember>()
        .iter()
        .find(|(_, display_info)| display_info.name().matches(target_name))
        .ok_or_else(|| "There is no crew member by that name.".to_string())?;

    if new_character == character {
        Err("You're already in control of them.".to_string())
    } else {
        Ok(CommandResult::ChangeControlled(new_character))
    }
}

fn trade(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let shopkeeper = world
        .query::<&Pos>()
        .with::<&Shopkeeper>()
        .iter()
        .filter(|(_, pos)| pos.is_in(area))
        .map(|(id, _)| id)
        .next()
        .ok_or_else(|| "There is no shopkeeper to trade with here.".to_string())?;
    command::action_result(Action::Trade(shopkeeper))
}

fn recruit_targets(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(&DisplayInfo, &Pos)>()
        .with::<&Recruitable>()
        .iter()
        .filter(|(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (display_info, _))| (display_info.name().base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}
