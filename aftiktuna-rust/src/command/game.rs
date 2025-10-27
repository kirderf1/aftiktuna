use crate::action::{Action, ForceDoorAction};
use crate::asset::GameAssets;
use crate::command::parse::{Parse, first_match_or};
use crate::command::{self, CommandResult};
use crate::core::area::{self, ShipControls, ShipState, ShipStatus};
use crate::core::behavior::{self, Character};
use crate::core::inventory::{self, Held};
use crate::core::item::ItemTypeId;
use crate::core::name::{Name, NameData, NameQuery};
use crate::core::position::{self, Blockage, Placement, PlacementQuery, Pos};
use crate::core::{self, CrewMember, FortunaChest, status};
use crate::game_loop::GameState;
use hecs::{Entity, Query, World};

mod combat;
mod dialogue;
mod item;

pub fn parse(input: &str, state: &GameState, assets: &GameAssets) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let area = world.get::<&Pos>(character).unwrap().get_area();

    let parse = Parse::new(input);
    first_match_or!(
        item::commands(&parse, state, assets),
        parse.literal("enter", |parse| {
            parse.match_against(
                targets_in_room::<&core::Door>(area, world, assets),
                |parse, door| parse.done_or_err(|| enter(door, character, world, assets)),
                |_| Err("There is no such door or path here to go through.".to_owned()),
            )
        }),
        parse.literal("force", |parse| {
            parse.match_against(
                targets_in_room::<&core::Door>(area, world, assets),
                |parse, door| parse.done_or_err(|| force(door, character, world, assets)),
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
        combat::commands(&parse, state.world.entity(state.controlled).unwrap(), &state.world, assets),
        dialogue::commands(&parse, state, assets),
        parse.literal("wait", |parse| {
            parse.done_or_err(|| command::action_result(Action::Wait))
        }),
        parse.literal("rest", |parse| parse.done_or_err(|| rest(world, character))),
        parse.literal("refuel", |parse| {
            first_match_or!(
                parse.literal("ship", |parse| parse.done_or_err(|| refuel_ship(state, assets)));
                parse.default_err()
            )
        }),
        parse.literal("launch", |parse| {
            first_match_or!(
                parse.literal("ship", |parse| parse.done_or_err(|| launch_ship(state, assets)));
                parse.default_err()
            )
        }),
        parse.literal("status", |parse| {
            parse.done_or_err(|| command::status(state, assets))
        }),
        parse.literal("check", |parse| {
            parse.match_against(
                check_item_targets(world, character, assets),
                |parse, item| parse.done_or_err(|| check(world, item, assets)),
                |input| Err(format!("There is no item by the name \"{input}\" here.")),
            )
        }),
        parse.literal("control", |parse| {
            parse.match_against(
                crew_character_targets(world, assets),
                |parse, target| parse.done_or_err(|| control(character, target)),
                |input| Err(format!("There is no crew member by the name \"{input}\".")),
            )
        }),
        parse.literal("open", |parse| {
            parse.match_against(
                fortuna_chest_targets(world, character, assets),
                |parse, target| parse.done_or_err(|| open(world, character, target, assets)),
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
        parse.literal("tame", |parse| {
            parse.match_against(
                combat::hostile_targets(world, character, assets).into_iter().flat_map(|(name, targets)| targets.into_iter().map(move |target| (name.clone(), target))),
                |parse, target| parse.done_or_err(|| tame(world, character, target, assets)),
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
        parse.literal("name", |parse| {
            parse.match_against(
                crew_targets_in_room(area, world, assets),
                |parse, target| parse.take_remaining(|name| give_name(world, character, target, name.to_owned(), assets)),
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        });
        parse.default_err()
    )
}

fn crew_character_targets(world: &World, assets: &GameAssets) -> Vec<(String, Entity)> {
    world
        .query::<NameQuery>()
        .with::<(&CrewMember, &Character)>()
        .iter()
        .map(|(entity, query)| (NameData::from_query(query, assets).base(), entity))
        .collect()
}

fn crew_targets_in_room(area: Entity, world: &World, assets: &GameAssets) -> Vec<(String, Entity)> {
    targets_in_room::<&CrewMember>(area, world, assets)
}

fn targets_in_room<Q: Query>(
    area: Entity,
    world: &World,
    assets: &GameAssets,
) -> Vec<(String, Entity)> {
    world
        .query::<&Pos>()
        .with::<Q>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(area))
        .flat_map(|(entity, _)| {
            super::entity_names(world.entity(entity).unwrap(), assets)
                .into_iter()
                .map(move |name| (name, entity))
        })
        .collect()
}

fn targets_by_proximity<Q: Query>(
    compare_pos: Pos,
    world: &World,
    assets: &GameAssets,
) -> Vec<(String, Entity)> {
    let mut targets_with_pos = world
        .query::<&Pos>()
        .with::<Q>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(compare_pos.get_area()))
        .flat_map(|(entity, &pos)| {
            super::entity_names(world.entity(entity).unwrap(), assets)
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

fn enter(
    door: Entity,
    character: Entity,
    world: &World,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_accessible_with_message(door, character, true, world, assets)?;

    command::crew_action(Action::EnterDoor(door))
}

fn force(
    door: Entity,
    character: Entity,
    world: &World,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_accessible_with_message(door, character, false, world, assets)?;

    command::action_result(ForceDoorAction {
        door,
        assisting: None,
    })
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
    if !behavior::is_safe(world, area) {
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

fn refuel_ship(state: &GameState, assets: &GameAssets) -> Result<CommandResult, String> {
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
                NameData::find(world, character, assets).definite()
            )
        })?;
    check_adjacent_accessible_with_message(ship_controls, character, world, assets)?;

    let status = world
        .get::<&ShipState>(state.ship_core)
        .map_err(|_| "The crew has no ship.".to_string())?
        .status;

    if !matches!(status, ShipStatus::NeedFuel(_)) {
        return Err("The ship is already refueled.".to_string());
    }
    if !inventory::is_holding(ItemTypeId::is_fuel_can, world, character) {
        return Err(format!(
            "{} needs a fuel can to refuel the ship.",
            NameData::find(world, character, assets).definite()
        ));
    }
    command::action_result(Action::Refuel)
}

fn launch_ship(state: &GameState, assets: &GameAssets) -> Result<CommandResult, String> {
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
                NameData::find(world, character, assets).definite()
            )
        })?;
    check_adjacent_accessible_with_message(ship_controls, character, world, assets)?;

    let status = world
        .get::<&ShipState>(state.ship_core)
        .map_err(|_| "The crew has no ship.".to_string())?
        .status;
    if matches!(status, ShipStatus::NeedFuel(_))
        && !inventory::is_holding(ItemTypeId::is_fuel_can, world, character)
    {
        return Err(format!(
            "{} needs a fuel can to launch the ship.",
            NameData::find(world, character, assets).definite()
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

fn fortuna_chest_targets(
    world: &World,
    character: Entity,
    assets: &GameAssets,
) -> Vec<(String, Entity)> {
    let character_pos = *world.get::<&Pos>(character).unwrap();
    world
        .query::<(NameQuery, &Pos)>()
        .with::<&FortunaChest>()
        .iter()
        .filter(|&(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (query, _))| (NameData::from_query(query, assets).base(), entity))
        .collect()
}

fn open(
    world: &World,
    character: Entity,
    chest: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(chest, character, world, assets)?;

    command::action_result(Action::OpenChest(chest))
}

fn check_item_targets(
    world: &World,
    character: Entity,
    assets: &GameAssets,
) -> Vec<(String, Entity)> {
    let mut targets = held_item_targets(world, character, assets);
    targets.extend(placed_item_targets(world, character, assets));
    targets
}

fn placed_item_targets(
    world: &World,
    character: Entity,
    assets: &GameAssets,
) -> Vec<(String, Entity)> {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    world
        .query::<(&Pos, NameQuery)>()
        .iter()
        .filter(|&(_, (pos, _))| pos.is_in(area))
        .map(|(entity, (_, query))| (NameData::from_query(query, assets).base(), entity))
        .collect()
}

fn held_item_targets(world: &World, holder: Entity, assets: &GameAssets) -> Vec<(String, Entity)> {
    world
        .query::<(&Held, NameQuery)>()
        .iter()
        .filter(|&(_, (held, _))| held.held_by(holder))
        .map(|(entity, (_, query))| (NameData::from_query(query, assets).base(), entity))
        .collect()
}

fn check(world: &World, item: Entity, assets: &GameAssets) -> Result<CommandResult, String> {
    Ok(CommandResult::Info(crate::CommandInfo::Message(
        core::item::description(world.entity(item).unwrap(), assets),
    )))
}

fn tame(
    world: &World,
    character: Entity,
    target: Entity,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(target, character, world, assets)?;

    if !inventory::is_holding(ItemTypeId::is_food_ration, world, character) {
        return Err(format!(
            "{} needs a food ration to tame.",
            NameData::find(world, character, assets).definite()
        ));
    }

    command::action_result(Action::Tame(target))
}

fn give_name(
    world: &World,
    character: Entity,
    target: Entity,
    name: String,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    check_adjacent_accessible_with_message(target, character, world, assets)?;

    if world.entity(target).unwrap().has::<Name>() {
        return Err(format!(
            "{} already has a name.",
            NameData::find(world, target, assets).definite()
        ));
    }

    command::action_result(Action::Name(target, name))
}

enum Inaccessible {
    NotHere,
    Blocked(Blockage),
}

impl Inaccessible {
    fn into_message(
        self,
        world: &World,
        character: Entity,
        target: Entity,
        assets: &GameAssets,
    ) -> String {
        match self {
            Inaccessible::NotHere => format!(
                "{} can not reach {} from here.",
                NameData::find(world, character, assets).definite(),
                NameData::find(world, target, assets).definite()
            ),
            Inaccessible::Blocked(blockage) => blockage.into_message(world, assets),
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
    assets: &GameAssets,
) -> Result<(), String> {
    check_accessible(target, character, can_push, world)
        .map_err(|inaccessible| inaccessible.into_message(world, character, target, assets))
}

fn check_adjacent_accessible_with_message(
    target: Entity,
    character: Entity,
    world: &World,
    assets: &GameAssets,
) -> Result<(), String> {
    check_adjacent_accessible(target, character, world)
        .map_err(|inaccessible| inaccessible.into_message(world, character, target, assets))
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
    let target_placement = Placement::from(
        world
            .query_one::<PlacementQuery>(target)
            .unwrap()
            .get()
            .ok_or(Inaccessible::NotHere)?,
    );
    if !character_pos.is_in(target_placement.area()) {
        return Err(Inaccessible::NotHere);
    }
    let target_pos = target_placement.get_adjacent_towards(character_pos);
    position::check_is_blocked(
        world,
        world.entity(character).unwrap(),
        character_pos,
        target_pos,
    )?;
    Ok(())
}
