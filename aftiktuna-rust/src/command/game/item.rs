use crate::action::Action;
use crate::command::parse::{first_match, first_match_or, Parse};
use crate::command::CommandResult;
use crate::core::inventory::Held;
use crate::core::item::{CanWield, FuelCan, Item, Keycard, Medkit};
use crate::core::name::{NameData, NameQuery};
use crate::core::position::Pos;
use crate::core::status::Health;
use crate::core::{BlockType, Door};
use crate::game_loop::GameState;
use crate::{command, core};
use hecs::{Entity, World};

pub fn commands(parse: &Parse, state: &GameState) -> Option<Result<CommandResult, String>> {
    first_match!(
        parse.literal("take", |parse| {
            first_match_or!(
                parse.literal("all", |parse| {
                    parse.done_or_err(|| take_all(state))
                });
                parse.take_remaining(|item_name| take(item_name, state))
            )
        }),
        parse.literal("give", |parse| {
            parse.match_against(
                super::crew_targets(&state.world),
                |parse, receiver| {
                    parse.take_remaining(|item_name| give(receiver, item_name, state))
                },
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
        parse.literal("wield", |parse| {
            parse.take_remaining(|item_name| wield(item_name, state))
        }),
        parse.literal("use", |parse| {
            parse.take_remaining(|item_name| use_item(state, item_name))
        }),
    )
}

fn take_all(state: &GameState) -> Result<CommandResult, String> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    if !state
        .world
        .query::<&Pos>()
        .with::<&Item>()
        .iter()
        .any(|(_, pos)| pos.is_in(character_pos.get_area()))
    {
        return Err("There are no items to take here.".to_string());
    }

    if !core::is_safe(&state.world, character_pos.get_area()) {
        return Err("You should take care of all foes here before taking all items.".to_string());
    }

    command::action_result(Action::TakeAll)
}

fn take(item_name: &str, state: &GameState) -> Result<CommandResult, String> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    let (item, name) = state
        .world
        .query::<(&Pos, NameQuery)>()
        .with::<&Item>()
        .iter()
        .map(|(item, (&pos, query))| (item, pos, NameData::from(query)))
        .filter(|(_, pos, name)| pos.is_in(character_pos.get_area()) && name.matches(item_name))
        .min_by_key(|(_, pos, _)| pos.distance_to(character_pos))
        .map(|(item, _, name)| (item, name))
        .ok_or_else(|| format!("There is no {item_name} here to pick up."))?;

    super::check_accessible_with_message(&state.world, state.controlled, item)?;

    command::action_result(Action::TakeItem(item, name))
}

fn give(receiver: Entity, item_name: &str, state: &GameState) -> Result<CommandResult, String> {
    if state.controlled == receiver {
        return Err(format!(
            "{} can't give an item to themselves.",
            NameData::find(&state.world, state.controlled).definite()
        ));
    }

    super::check_adjacent_accessible_with_message(&state.world, state.controlled, receiver)?;

    state
        .world
        .query::<(NameQuery, &Held)>()
        .with::<&Item>()
        .iter()
        .filter(|&(_, (query, held))| {
            NameData::from(query).matches(item_name) && held.held_by(state.controlled)
        })
        .min_by_key(|(_, (_, held))| held.is_in_hand())
        .map_or_else(
            || {
                Err(format!(
                    "{} has no {} to give.",
                    NameData::find(&state.world, state.controlled).definite(),
                    item_name,
                ))
            },
            |(item, _)| command::action_result(Action::GiveItem(item, receiver)),
        )
}

fn wield(item_name: &str, state: &GameState) -> Result<CommandResult, String> {
    if state
        .world
        .query::<(NameQuery, &Held)>()
        .into_iter()
        .any(|(_, (query, held))| {
            NameData::from(query).matches(item_name)
                && held.held_by(state.controlled)
                && held.is_in_hand()
        })
    {
        return Err(format!(
            "{} is already wielding a {}.",
            NameData::find(&state.world, state.controlled).definite(),
            item_name
        ));
    }

    if let Some((item, name)) =
        wieldable_item_in_inventory(item_name, &state.world, state.controlled)
    {
        return command::action_result(Action::Wield(item, name));
    }

    if let Some((item, name)) =
        wieldable_item_from_ground(item_name, &state.world, state.controlled)
    {
        super::check_accessible_with_message(&state.world, state.controlled, item)?;

        return command::action_result(Action::Wield(item, name));
    }
    Err(format!(
        "There is no {} that {} can wield.",
        item_name,
        NameData::find(&state.world, state.controlled).definite()
    ))
}

fn wieldable_item_in_inventory(
    item_name: &str,
    world: &World,
    character: Entity,
) -> Option<(Entity, NameData)> {
    world
        .query::<(NameQuery, &Held)>()
        .with::<&CanWield>()
        .with::<&Item>()
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
        .with::<&CanWield>()
        .with::<&Item>()
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
        .ok_or_else(|| format!("No held item by the name \"{item_name}\"."))?
        .0;

    if world.get::<&FuelCan>(item).is_ok() {
        super::refuel_ship(state)
    } else if world.get::<&Medkit>(item).is_ok() {
        if !world.get::<&Health>(character).unwrap().is_hurt() {
            return Err(format!(
                "{} is not hurt, and does not need to use the medkit.",
                NameData::find(world, character).definite()
            ));
        }
        command::action_result(Action::UseMedkit(item))
    } else if world.get::<&Keycard>(item).is_ok() {
        let area = world.get::<&Pos>(character).unwrap().get_area();
        let (door, _) = world
            .query::<(&Pos, &Door)>()
            .into_iter()
            .find(|(_, (door_pos, door))| {
                door_pos.is_in(area)
                    && world
                        .get::<&BlockType>(door.door_pair)
                        .map_or(false, |block_type| BlockType::Locked.eq(&block_type))
            })
            .ok_or_else(|| {
                "There is no accessible door here that requires a keycard.".to_string()
            })?;

        command::crew_action(Action::EnterDoor(door))
    } else if world.get::<&CanWield>(item).is_ok() {
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
