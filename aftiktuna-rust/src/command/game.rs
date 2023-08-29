use crate::action::trade::Shopkeeper;
use crate::action::{combat, door, Action, CrewMember, FortunaChest, Recruitable};
use crate::area::Ship;
use crate::command::parse::Parse;
use crate::command::CommandResult;
use crate::core::inventory::Held;
use crate::core::position::{Blockage, Pos};
use crate::core::status::Health;
use crate::core::{inventory, item, position, status, GameState};
use crate::view::name::{NameData, NameQuery};
use crate::{command, core};
use hecs::{Entity, World};

pub fn parse(input: &str, state: &GameState) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    Parse::new(input)
        .literal("take", |parse| {
            parse
                .literal("all", |parse| {
                    parse.done_or_err(|| take_all(world, character))
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
            parse.take_remaining(|item_name| use_item(state, item_name))
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
                .literal("ship", |parse| parse.done_or_err(|| launch_ship(state)))
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
        .literal("open", |parse| {
            parse.match_against(
                fortuna_chest_targets(world, character),
                |parse, target| parse.done_or_err(|| open(world, character, target)),
                |input| Err(format!("\"{}\" not a valid target", input)),
            )
        })
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}

fn take_all(world: &World, character: Entity) -> Result<CommandResult, String> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    if !world
        .query::<&Pos>()
        .with::<&item::Item>()
        .iter()
        .any(|(_, pos)| pos.is_in(character_pos.get_area()))
    {
        return Err("There are no items to take here.".to_string());
    }

    if !core::is_safe(world, character_pos.get_area()) {
        return Err("You should take care of all foes here before taking all items.".to_string());
    }

    command::action_result(Action::TakeAll)
}

fn crew_targets(world: &World) -> Vec<(String, Entity)> {
    world
        .query::<NameQuery>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, query)| (NameData::from(query).base().to_lowercase(), entity))
        .collect()
}

fn take(item_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    let (item, name) = world
        .query::<(&Pos, NameQuery)>()
        .with::<&item::Item>()
        .iter()
        .map(|(item, (&pos, query))| (item, pos, NameData::from(query)))
        .filter(|(_, pos, name)| pos.is_in(character_pos.get_area()) && name.matches(item_name))
        .min_by_key(|(_, pos, _)| pos.distance_to(character_pos))
        .map(|(item, _, name)| (item, name))
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
        .query::<(NameQuery, &Held)>()
        .with::<&item::Item>()
        .iter()
        .filter(|&(_, (query, held))| {
            NameData::from(query).matches(item_name) && held.held_by(character)
        })
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
        .query::<(NameQuery, &Held)>()
        .into_iter()
        .any(|(_, (query, held))| {
            NameData::from(query).matches(item_name) && held.held_by(character) && held.is_in_hand()
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
        .query::<(NameQuery, &Held)>()
        .with::<&item::CanWield>()
        .with::<&item::Item>()
        .iter()
        .map(|(item, (query, held))| (item, NameData::from(query), held))
        .find(|(_, name, held)| name.matches(item_name) && held.is_in_inventory(character))
        .map(|(item, name, _)| (item, name))
}

fn wieldable_item_from_ground(
    item_name: &str,
    world: &World,
    character: Entity,
) -> Option<(Entity, NameData)> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(&Pos, NameQuery)>()
        .with::<&item::CanWield>()
        .with::<&item::Item>()
        .iter()
        .map(|(item, (&pos, query))| (item, pos, NameData::from(query)))
        .filter(|(_, pos, name)| pos.is_in(character_pos.get_area()) && name.matches(item_name))
        .min_by_key(|(_, pos, _)| pos.distance_to(character_pos))
        .map(|(item, _, name)| (item, name))
}

fn use_item(state: &GameState, item_name: &str) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let item = world
        .query::<(&Held, NameQuery)>()
        .iter()
        .filter(|&(_, (held, query))| {
            held.held_by(character) && NameData::from(query).matches(item_name)
        })
        .max_by_key(|(_, (held, _))| held.is_in_hand())
        .ok_or_else(|| format!("No held item by the name \"{}\".", item_name))?
        .0;

    if world.get::<&item::FuelCan>(item).is_ok() {
        launch_ship(state)
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
        .query::<(&Pos, NameQuery)>()
        .with::<&door::Door>()
        .iter()
        .find(|&(_, (pos, query))| pos.is_in(area) && NameData::from(query).matches(door_name))
        .map(|(door, _)| door)
        .ok_or_else(|| "There is no such door or path here to go through.".to_string())?;

    check_accessible_with_message(world, character, door)?;

    command::crew_action(Action::EnterDoor(door))
}

fn force(door_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let door = world
        .query::<(&Pos, NameQuery)>()
        .with::<&door::Door>()
        .iter()
        .find(|&(_, (pos, query))| pos.is_in(area) && NameData::from(query).matches(door_name))
        .map(|(door, _)| door)
        .ok_or_else(|| "There is no such door here.".to_string())?;

    check_accessible_with_message(world, character, door)?;

    command::action_result(Action::ForceDoor(door))
}

fn attack_any(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let foes = world
        .query::<&Pos>()
        .with::<&combat::IsFoe>()
        .iter()
        .filter(|(_, pos)| pos.is_in(area))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    if foes.is_empty() {
        Err("There is no appropriate target to attack here.".to_string())
    } else {
        command::action_result(Action::Attack(foes))
    }
}

fn attack(target_name: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    let pos = *world.get::<&Pos>(character).unwrap();
    let targets = world
        .query::<(&Pos, NameQuery)>()
        .with::<&combat::IsFoe>()
        .iter()
        .filter(|&(_, (target_pos, query))| {
            target_pos.is_in(pos.get_area()) && NameData::from(query).matches(target_name)
        })
        .map(|(target, (&pos, _))| (target, pos))
        .collect::<Vec<_>>();

    if targets.is_empty() {
        return Err("There is no such target here.".to_string());
    }

    let target_access = targets
        .iter()
        .map(|&(entity, pos)| {
            (
                check_adjacent_accessible_with_message(world, character, entity),
                pos,
            )
        })
        .collect::<Vec<_>>();
    if target_access.iter().all(|(result, _)| result.is_err()) {
        return Err(target_access
            .into_iter()
            .min_by_key(|&(_, target_pos)| pos.distance_to(target_pos))
            .unwrap()
            .0
            .unwrap_err());
    }

    command::action_result(Action::Attack(
        targets.into_iter().map(|(entity, _)| entity).collect(),
    ))
}

fn rest(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    if !core::is_safe(world, area) {
        return Err("This area is not safe to rest in.".to_string());
    }

    let need_rest = world
        .query::<(&status::Stamina, &Pos)>()
        .with::<&CrewMember>()
        .iter()
        .any(|(_, (stamina, pos))| pos.is_in(area) && stamina.need_rest());

    if !need_rest {
        return Err("The crew is already rested.".to_string());
    }

    command::action_result(Action::Rest(true))
}

fn launch_ship(state: &GameState) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    if state.locations.is_at_fortuna() {
        return Err("You are already at your final destination. You should find the fortuna chest before leaving!".to_string());
    }

    let area = world.get::<&Pos>(character).unwrap().get_area();
    if !inventory::is_holding::<&item::FuelCan>(world, character) {
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
        .query::<NameQuery>()
        .with::<&CrewMember>()
        .iter()
        .find(|&(_, query)| NameData::from(query).matches(target_name))
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
        .query::<(NameQuery, &Pos)>()
        .with::<&Recruitable>()
        .iter()
        .filter(|(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (query, _))| (NameData::from(query).base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

fn fortuna_chest_targets(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(NameQuery, &Pos)>()
        .with::<&FortunaChest>()
        .iter()
        .filter(|&(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (query, _))| (NameData::from(query).base().to_lowercase(), entity))
        .collect()
}

fn open(world: &World, character: Entity, chest: Entity) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(world, character, chest)?;

    command::action_result(Action::OpenChest(chest))
}

enum Inaccessible {
    NotHere,
    Blocked(Blockage),
}

impl Inaccessible {
    fn into_message(self, world: &World, character: Entity, target: Entity) -> String {
        match self {
            Inaccessible::NotHere => format!(
                "{} can not reach {} from here.",
                NameData::find(world, character).definite(),
                NameData::find(world, target).definite()
            ),
            Inaccessible::Blocked(blockage) => blockage.into_message(),
        }
    }
}

impl From<Blockage> for Inaccessible {
    fn from(value: Blockage) -> Self {
        Inaccessible::Blocked(value)
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
    position::check_is_blocked(world, character, character_pos, target_pos)?;

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
    position::check_is_blocked(world, character, character_pos, target_pos)?;
    Ok(())
}
