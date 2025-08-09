use crate::action::Action;
use crate::command::CommandResult;
use crate::command::parse::{Parse, first_match_or};
use crate::core::area::{ShipControls, ShipState, ShipStatus};
use crate::core::inventory::Held;
use crate::core::item::{FoodRation, FuelCan};
use crate::core::name::{Name, NameData, NameQuery};
use crate::core::position::{Blockage, Pos};
use crate::core::store::Shopkeeper;
use crate::core::{Character, CrewMember, FortunaChest, area, inventory, position, status};
use crate::game_loop::GameState;
use crate::{command, core};
use hecs::{Entity, Query, World};

mod combat;
mod dialogue;
mod item;

pub fn parse(input: &str, state: &GameState) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let area = world.get::<&Pos>(character).unwrap().get_area();

    let parse = Parse::new(input);
    first_match_or!(
        item::commands(&parse, state),
        parse.literal("enter", |parse| {
            parse.match_against(
                targets_in_room::<&core::Door>(area, world),
                |parse, door| parse.done_or_err(|| enter(door, character, world)),
                |_| Err("There is no such door or path here to go through.".to_owned()),
            )
        }),
        parse.literal("force", |parse| {
            parse.match_against(
                targets_in_room::<&core::Door>(area, world),
                |parse, door| parse.done_or_err(|| force(door, character, world)),
                |_| Err("There is no such door here.".to_owned()),
            )
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
            parse.done_or_err(|| command::status(state))
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
                crew_character_targets(world),
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
        }),
        parse.literal("tame", |parse| {
            parse.match_against(
                combat::hostile_targets(world, character).into_iter().flat_map(|(name, targets)| targets.into_iter().map(move |target| (name.clone(), target))),
                |parse, target| parse.done_or_err(|| tame(world, character, target)),
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
        parse.literal("name", |parse| {
            parse.match_against(
                crew_targets_in_room(area, world),
                |parse, target| parse.take_remaining(|name| give_name(world, character, target, name.to_owned())),
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        });
        parse.default_err()
    )
}

fn crew_character_targets(world: &World) -> Vec<(String, Entity)> {
    world
        .query::<NameQuery>()
        .with::<(&CrewMember, &Character)>()
        .iter()
        .map(|(entity, query)| (NameData::from(query).base().to_lowercase(), entity))
        .collect()
}

fn crew_targets_in_room(area: Entity, world: &World) -> Vec<(String, Entity)> {
    targets_in_room::<&CrewMember>(area, world)
}

fn targets_in_room<Q: Query>(area: Entity, world: &World) -> Vec<(String, Entity)> {
    world
        .query::<&Pos>()
        .with::<Q>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(area))
        .flat_map(|(entity, _)| {
            super::entity_names(world.entity(entity).unwrap())
                .into_iter()
                .map(move |name| (name, entity))
        })
        .collect()
}

fn targets_by_proximity<Q: Query>(compare_pos: Pos, world: &World) -> Vec<(String, Entity)> {
    let mut targets_with_pos = world
        .query::<&Pos>()
        .with::<Q>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(compare_pos.get_area()))
        .flat_map(|(entity, &pos)| {
            super::entity_names(world.entity(entity).unwrap())
                .into_iter()
                .map(move |name| (name, entity, pos))
        })
        .collect::<Vec<_>>();

    targets_with_pos.sort_by_key(|(_, _, pos)| pos.distance_to(compare_pos));
    targets_with_pos
        .into_iter()
        .map(|(name, entity, _)| (name, entity))
        .collect()
}

fn enter(door: Entity, character: Entity, world: &World) -> Result<CommandResult, String> {
    check_accessible_with_message(door, character, true, world)?;

    command::crew_action(Action::EnterDoor(door))
}

fn force(door: Entity, character: Entity, world: &World) -> Result<CommandResult, String> {
    check_accessible_with_message(door, character, false, world)?;

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
    let ship_controls = world
        .query::<&Pos>()
        .with::<&ShipControls>()
        .iter()
        .find(|(_, pos)| pos.is_in(area) && area::is_ship(area, world))
        .map(|(entity, _)| entity)
        .ok_or_else(|| {
            format!(
                "{} needs to be in the ship control room in order to refuel it.",
                NameData::find(world, character).definite()
            )
        })?;
    check_adjacent_accessible_with_message(ship_controls, character, world)?;

    let status = world
        .get::<&ShipState>(state.ship_core)
        .map_err(|_| "The crew has no ship.".to_string())?
        .status;

    if !matches!(status, ShipStatus::NeedFuel(_)) {
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
    if state.generation_state.is_at_fortuna() {
        return Err("You are already at your final destination. You should find the fortuna chest before leaving!".to_string());
    }

    let area = world.get::<&Pos>(character).unwrap().get_area();
    let ship_controls = world
        .query::<&Pos>()
        .with::<&ShipControls>()
        .iter()
        .find(|(_, pos)| pos.is_in(area) && area::is_ship(area, world))
        .map(|(entity, _)| entity)
        .ok_or_else(|| {
            format!(
                "{} needs to be in the ship control room in order to launch it.",
                NameData::find(world, character).definite()
            )
        })?;
    check_adjacent_accessible_with_message(ship_controls, character, world)?;

    let status = world
        .get::<&ShipState>(state.ship_core)
        .map_err(|_| "The crew has no ship.".to_string())?
        .status;
    if matches!(status, ShipStatus::NeedFuel(_))
        && !inventory::is_holding::<&FuelCan>(world, character)
    {
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

    check_adjacent_accessible_with_message(shopkeeper, character, world)?;

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
    check_adjacent_accessible_with_message(chest, character, world)?;

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
    Ok(CommandResult::Info(crate::CommandInfo::Message(
        core::item::description(world.entity(item).unwrap()),
    )))
}

fn tame(world: &World, character: Entity, target: Entity) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(target, character, world)?;

    if !inventory::is_holding::<&FoodRation>(world, character) {
        return Err(format!(
            "{} needs a food ration to tame.",
            NameData::find(world, character).definite()
        ));
    }

    command::action_result(Action::Tame(target))
}

fn give_name(
    world: &World,
    character: Entity,
    target: Entity,
    name: String,
) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(target, character, world)?;

    if world.entity(target).unwrap().has::<Name>() {
        return Err(format!(
            "{} already has a name.",
            NameData::find(world, target).definite()
        ));
    }

    command::action_result(Action::Name(target, name))
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
            Inaccessible::Blocked(blockage) => blockage.into_message(world),
        }
    }
}

impl From<Blockage> for Inaccessible {
    fn from(value: Blockage) -> Self {
        Inaccessible::Blocked(value)
    }
}

fn check_accessible_with_message(
    target: Entity,
    character: Entity,
    can_push: bool,
    world: &World,
) -> Result<(), String> {
    check_accessible(target, character, can_push, world)
        .map_err(|inaccessible| inaccessible.into_message(world, character, target))
}

fn check_adjacent_accessible_with_message(
    target: Entity,
    character: Entity,
    world: &World,
) -> Result<(), String> {
    check_adjacent_accessible(target, character, world)
        .map_err(|inaccessible| inaccessible.into_message(world, character, target))
}

fn check_accessible(
    target: Entity,
    character: Entity,
    can_push: bool,
    world: &World,
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
    if let Err(blockage) = position::check_is_blocked(
        world,
        world.entity(character).unwrap(),
        character_pos,
        target_pos,
    ) {
        let is_inaccessible = match blockage {
            Blockage::Hostile(_) => true,
            Blockage::TakesSpace(_) => !can_push,
        };
        if is_inaccessible {
            return Err(Inaccessible::Blocked(blockage));
        }
    }

    Ok(())
}

fn check_adjacent_accessible(
    target: Entity,
    character: Entity,
    world: &World,
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
    position::check_is_blocked(
        world,
        world.entity(character).unwrap(),
        character_pos,
        target_pos,
    )?;
    Ok(())
}
