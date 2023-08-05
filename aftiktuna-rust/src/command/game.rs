use crate::action::item::{is_holding, Held};
use crate::action::trade::Shopkeeper;
use crate::action::{combat, door, Action, CrewMember, Recruitable};
use crate::area::Ship;
use crate::command::parse::Parse;
use crate::command::CommandResult;
use crate::position::Pos;
use crate::status::Health;
use crate::view::NameData;
use crate::{command, item, position, status};
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
        .literal("use", |parse| {
            parse.take_remaining(|item_name| use_item(world, character, item_name))
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
                |parse, target| parse.done_or_err(|| recruit(world, character, target)),
                |input| Err(format!("\"{}\" not a valid recruitment target", input)),
            )
        })
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}

fn crew_targets(world: &World) -> Vec<(String, Entity)> {
    world
        .query::<&NameData>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, name)| (name.base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

fn take(item_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    let (item, name) = world
        .query::<(&Pos, &NameData)>()
        .with::<&item::Item>()
        .iter()
        .filter(|(_, (pos, name))| pos.is_in(character_pos.get_area()) && name.matches(item_name))
        .min_by_key(|(_, (pos, _))| pos.distance_to(character_pos))
        .map(|(item, (_, name))| (item, name.clone()))
        .ok_or_else(|| format!("There is no {} here to pick up.", item_name))?;

    check_accessible_with_message(world, character, item)?;

    command::action_result(Action::TakeItem(item, name))
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

    check_adjacent_accessible_with_message(world, character, receiver)?;

    world
        .query::<(&NameData, &Held)>()
        .with::<&item::Item>()
        .iter()
        .filter(|(_, (name, held))| name.matches(item_name) && held.held_by(character))
        .min_by_key(|(_, (_, held))| held.is_in_hand())
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
    if world
        .query::<(&NameData, &Held)>()
        .into_iter()
        .any(|(_, (name, held))| {
            name.matches(item_name) && held.held_by(character) && held.is_in_hand()
        })
    {
        return Err(format!(
            "{} is already wielding a {}.",
            NameData::find(world, character).definite(),
            item_name
        ));
    }

    if let Some((item, name)) = wieldable_item_in_inventory(item_name, world, character) {
        return command::action_result(Action::Wield(item, name));
    }

    if let Some((item, name)) = wieldable_item_from_ground(item_name, world, character) {
        check_accessible_with_message(world, character, item)?;

        return command::action_result(Action::Wield(item, name));
    }
    Err(format!(
        "There is no {} that {} can wield.",
        item_name,
        NameData::find(world, character).definite()
    ))
}

fn wieldable_item_in_inventory(
    item_name: &str,
    world: &World,
    character: Entity,
) -> Option<(Entity, NameData)> {
    world
        .query::<(&NameData, &Held)>()
        .with::<&item::CanWield>()
        .with::<&item::Item>()
        .iter()
        .find(|(_, (name, held))| name.matches(item_name) && held.is_in_inventory(character))
        .map(|(item, (name, _))| (item, name.clone()))
}

fn wieldable_item_from_ground(
    item_name: &str,
    world: &World,
    character: Entity,
) -> Option<(Entity, NameData)> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(&Pos, &NameData)>()
        .with::<&item::CanWield>()
        .with::<&item::Item>()
        .iter()
        .filter(|(_, (pos, name))| pos.is_in(character_pos.get_area()) && name.matches(item_name))
        .min_by_key(|(_, (pos, _))| pos.distance_to(character_pos))
        .map(|(item, (_, name))| (item, name.clone()))
}

fn use_item(world: &World, character: Entity, item_name: &str) -> Result<CommandResult, String> {
    let item = world
        .query::<(&Held, &NameData)>()
        .iter()
        .filter(|(_, (held, name_data))| held.held_by(character) && name_data.matches(item_name))
        .max_by_key(|(_, (held, _))| held.is_in_hand())
        .ok_or_else(|| format!("No held item by the name \"{}\".", item_name))?
        .0;

    if world.get::<&item::FuelCan>(item).is_ok() {
        launch_ship(world, character)
    } else if world.get::<&item::Medkit>(item).is_ok() {
        if !world.get::<&Health>(character).unwrap().is_hurt() {
            return Err(format!(
                "{} is not hurt, and does not need to use the medkit.",
                NameData::find(world, character).definite()
            ));
        }
        command::action_result(Action::UseMedkit(item))
    } else if world.get::<&item::Keycard>(item).is_ok() {
        let area = world.get::<&Pos>(character).unwrap().get_area();
        let (door, _) = world
            .query::<(&Pos, &door::Door)>()
            .into_iter()
            .find(|(_, (door_pos, door))| {
                door_pos.is_in(area)
                    && world
                        .get::<&door::BlockType>(door.door_pair)
                        .map_or(false, |block_type| door::BlockType::Locked.eq(&block_type))
            })
            .ok_or_else(|| {
                "There is no accessible door here that requires a keycard.".to_string()
            })?;

        command::crew_action(Action::EnterDoor(door))
    } else if world.get::<&item::CanWield>(item).is_ok() {
        if world
            .get::<&Held>(item)
            .map_or(false, |held| held.is_in_hand())
        {
            Err(format!(
                "{} is already being held.",
                NameData::find(world, item).definite()
            ))
        } else {
            command::action_result(Action::Wield(item, NameData::find(world, item)))
        }
    } else {
        Err("The item can not be used in any meaningful way.".to_string())
    }
}

fn enter(door_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let door = world
        .query::<(&Pos, &NameData)>()
        .with::<&door::Door>()
        .iter()
        .find(|(_, (pos, name))| pos.is_in(area) && name.matches(door_name))
        .map(|(door, _)| door)
        .ok_or_else(|| "There is no such door or path here to go through.".to_string())?;

    check_accessible_with_message(world, character, door)?;

    command::crew_action(Action::EnterDoor(door))
}

fn force(door_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let door = world
        .query::<(&Pos, &NameData)>()
        .with::<&door::Door>()
        .iter()
        .find(|(_, (pos, name))| pos.is_in(area) && name.matches(door_name))
        .map(|(door, _)| door)
        .ok_or_else(|| "There is no such door here.".to_string())?;

    check_accessible_with_message(world, character, door)?;

    command::action_result(Action::ForceDoor(door))
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
    let target = world
        .query::<(&Pos, &NameData)>()
        .with::<&combat::IsFoe>()
        .iter()
        .find(|(_, (pos, name))| pos.is_in(area) && name.matches(target_name))
        .map(|(target, _)| target)
        .ok_or_else(|| "There is no such target here.".to_string())?;

    check_adjacent_accessible_with_message(world, character, target)?;

    command::action_result(Action::Attack(target))
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
    if !is_holding::<item::FuelCan>(world, character) {
        return Err(format!(
            "{} needs a fuel can to launch the ship.",
            NameData::find(world, character).definite()
        ));
    }
    world.get::<&Ship>(area).map_err(|_| {
        format!(
            "{} needs to be in the ship in order to launch it.",
            NameData::find(world, character).definite()
        )
    })?;
    command::action_result(Action::Launch)
}

fn control(world: &World, character: Entity, target_name: &str) -> Result<CommandResult, String> {
    let (new_character, _) = world
        .query::<&NameData>()
        .with::<&CrewMember>()
        .iter()
        .find(|(_, name)| name.matches(target_name))
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

    check_adjacent_accessible_with_message(world, character, shopkeeper)?;

    command::action_result(Action::Trade(shopkeeper))
}

fn recruit(world: &World, character: Entity, target: Entity) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(world, character, target)?;

    command::action_result(Action::Recruit(target))
}

fn recruit_targets(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(&NameData, &Pos)>()
        .with::<&Recruitable>()
        .iter()
        .filter(|(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (name, _))| (name.base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

enum Inaccessible {
    NotHere,
    Blocked,
}

impl Inaccessible {
    fn into_message(self, world: &World, character: Entity, target: Entity) -> String {
        match self {
            Inaccessible::NotHere => format!(
                "{} can not reach {} from here.",
                NameData::find(world, character).definite(),
                NameData::find(world, target).definite()
            ),
            Inaccessible::Blocked => "Something is in the way.".to_string(),
        }
    }
}

fn check_accessible_with_message(
    world: &World,
    character: Entity,
    target: Entity,
) -> Result<(), String> {
    check_accessible(world, character, target)
        .map_err(|inaccessible| inaccessible.into_message(world, character, target))
}

fn check_adjacent_accessible_with_message(
    world: &World,
    character: Entity,
    target: Entity,
) -> Result<(), String> {
    check_adjacent_accessible(world, character, target)
        .map_err(|inaccessible| inaccessible.into_message(world, character, target))
}

fn check_accessible(world: &World, character: Entity, target: Entity) -> Result<(), Inaccessible> {
    let character_pos = *world
        .get::<&Pos>(character)
        .map_err(|_| Inaccessible::NotHere)?;
    let target_pos = *world
        .get::<&Pos>(target)
        .map_err(|_| Inaccessible::NotHere)?;
    if !character_pos.is_in(target_pos.get_area()) {
        return Err(Inaccessible::NotHere);
    }
    if position::is_blocked(world, character, character_pos, target_pos) {
        return Err(Inaccessible::Blocked);
    }
    Ok(())
}

fn check_adjacent_accessible(
    world: &World,
    character: Entity,
    target: Entity,
) -> Result<(), Inaccessible> {
    let character_pos = *world
        .get::<&Pos>(character)
        .map_err(|_| Inaccessible::NotHere)?;
    let target_pos = *world
        .get::<&Pos>(target)
        .map_err(|_| Inaccessible::NotHere)?;
    if !character_pos.is_in(target_pos.get_area()) {
        return Err(Inaccessible::NotHere);
    }
    let target_pos = target_pos.get_adjacent_towards(character_pos);
    if position::is_blocked(world, character, character_pos, target_pos) {
        return Err(Inaccessible::Blocked);
    }
    Ok(())
}
