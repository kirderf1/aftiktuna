use crate::action::trade::Shopkeeper;
use crate::action::{door, Action, FortunaChest};
use crate::command::parse::{first_match_or, Parse};
use crate::command::CommandResult;
use crate::core::area::{Ship, ShipStatus};
use crate::core::inventory::Held;
use crate::core::item::FuelCan;
use crate::core::position::{Blockage, Pos};
use crate::core::{area, inventory, position, status, CrewMember, GameState};
use crate::view::name::{NameData, NameQuery};
use crate::{command, core};
use hecs::{Entity, World};

mod combat;
mod dialogue;
mod item;

pub fn parse(input: &str, state: &GameState) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let parse = Parse::new(input);
    first_match_or!(
        item::commands(&parse, state),
        parse.literal("enter", |parse| {
            parse.take_remaining(|door_name| enter(door_name, world, character))
        }),
        parse.literal("force", |parse| {
            parse.take_remaining(|door_name| force(door_name, world, character))
        }),
        parse.literal("go to", |parse|
            first_match_or!(
                parse.literal("ship", |parse|
                    parse.done_or_err(|| go_to_ship(world, character))
                );
                parse.default_err()
            )
        ),
        combat::commands(&parse, state),
        dialogue::commands(&parse, state),
        parse.literal("wait", |parse| {
            parse.done_or_err(|| command::action_result(Action::Wait))
        }),
        parse.literal("rest", |parse| parse.done_or_err(|| rest(world, character))),
        parse.literal("refuel", |parse| {
            first_match_or!(
                parse.literal("ship", |parse| parse.done_or_err(|| refuel_ship(state)));
                parse.default_err()
            )
        }),
        parse.literal("launch", |parse| {
            first_match_or!(
                parse.literal("ship", |parse| parse.done_or_err(|| launch_ship(state)));
                parse.default_err()
            )
        }),
        parse.literal("status", |parse| {
            parse.done_or_err(|| command::status(world, character))
        }),
        parse.literal("check", |parse| {
            parse.match_against(
                check_item_targets(world, character),
                |parse, item| parse.done_or_err(|| check(world, item)),
                |input| Err(format!("There is no item by the name \"{input}\" here.")),
            )
        }),
        parse.literal("control", |parse| {
            parse.match_against(
                crew_targets(world),
                |parse, target| parse.done_or_err(|| control(character, target)),
                |input| Err(format!("There is no crew member by the name \"{input}\".")),
            )
        }),
        parse.literal("trade", |parse| {
            parse.done_or_err(|| trade(world, character))
        }),
        parse.literal("open", |parse| {
            parse.match_against(
                fortuna_chest_targets(world, character),
                |parse, target| parse.done_or_err(|| open(world, character, target)),
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        });
        parse.default_err()
    )
}

fn crew_targets(world: &World) -> Vec<(String, Entity)> {
    world
        .query::<NameQuery>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, query)| (NameData::from(query).base().to_lowercase(), entity))
        .collect()
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

    command::action_result(Action::ForceDoor(door, false))
}

fn go_to_ship(world: &World, character: Entity) -> Result<CommandResult, String> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    if area::is_ship(area, world) {
        return Err("You are already at the ship.".to_string());
    }
    command::crew_action(Action::GoToShip)
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

fn refuel_ship(state: &GameState) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;

    let area = world.get::<&Pos>(character).unwrap().get_area();
    let need_fuel = world
        .get::<&Ship>(area)
        .map(|ship| matches!(ship.status, ShipStatus::NeedFuel(_)))
        .map_err(|_| {
            format!(
                "{} needs to be in the ship in order to refuel it.",
                NameData::find(world, character).definite()
            )
        })?;
    if !need_fuel {
        return Err("The ship is already refueled.".to_string());
    }
    if !inventory::is_holding::<&FuelCan>(world, character) {
        return Err(format!(
            "{} needs a fuel can to refuel the ship.",
            NameData::find(world, character).definite()
        ));
    }
    command::action_result(Action::Refuel)
}

fn launch_ship(state: &GameState) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    if state.locations.is_at_fortuna() {
        return Err("You are already at your final destination. You should find the fortuna chest before leaving!".to_string());
    }

    let area = world.get::<&Pos>(character).unwrap().get_area();
    let need_fuel = world
        .get::<&Ship>(area)
        .map(|ship| matches!(ship.status, ShipStatus::NeedFuel(_)))
        .map_err(|_| {
            format!(
                "{} needs to be in the ship in order to launch it.",
                NameData::find(world, character).definite()
            )
        })?;
    if need_fuel && !inventory::is_holding::<&FuelCan>(world, character) {
        return Err(format!(
            "{} needs a fuel can to launch the ship.",
            NameData::find(world, character).definite()
        ));
    }
    command::action_result(Action::Launch)
}

fn control(character: Entity, target: Entity) -> Result<CommandResult, String> {
    if target == character {
        Err("You're already in control of them.".to_string())
    } else {
        Ok(CommandResult::ChangeControlled(target))
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

fn check_item_targets(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let mut targets = held_item_targets(world, character);
    targets.extend(placed_item_targets(world, character));
    targets
}

fn placed_item_targets(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    world
        .query::<(&Pos, NameQuery)>()
        .iter()
        .filter(|&(_, (pos, _))| pos.is_in(area))
        .map(|(entity, (_, query))| (NameData::from(query).base().to_lowercase(), entity))
        .collect()
}

fn held_item_targets(world: &World, holder: Entity) -> Vec<(String, Entity)> {
    world
        .query::<(&Held, NameQuery)>()
        .iter()
        .filter(|&(_, (held, _))| held.held_by(holder))
        .map(|(entity, (_, query))| (NameData::from(query).base().to_lowercase(), entity))
        .collect()
}

fn check(world: &World, item: Entity) -> Result<CommandResult, String> {
    Ok(CommandResult::Info(core::item::description(
        world.entity(item).unwrap(),
    )))
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
