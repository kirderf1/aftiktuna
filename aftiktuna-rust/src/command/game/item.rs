use crate::action::Action;
use crate::action::item::{SearchAction, UseAction};
use crate::asset::NounDataMap;
use crate::command::parse::{Parse, first_match, first_match_or};
use crate::command::{self, CommandResult};
use crate::core::behavior;
use crate::core::inventory::{Container, Held};
use crate::core::item::{CanWield, ItemType};
use crate::core::name::NameData;
use crate::core::position::Pos;
use crate::core::status::Health;
use crate::game_loop::GameState;
use hecs::{Entity, World};

pub fn commands(
    parse: &Parse,
    state: &GameState,
    noun_map: &NounDataMap,
) -> Option<Result<CommandResult, String>> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    first_match!(
        parse.literal("take", |parse| {
            first_match_or!(
                parse.literal("all", |parse| {
                    parse.done_or_err(|| take_all(state))
                });
                parse.match_against(
                    super::targets_by_proximity::<&ItemType>(character_pos, &state.world, noun_map),
                    |parse, item| parse.done_or_err(|| take(item, state, noun_map)),
                    |input| Err(format!("There is no {input} here to pick up.")),
                )
            )
        }),
        parse.literal("search", |parse| {
            parse.match_against(
                super::targets_in_room::<&Container>(
                    character_pos.get_area(),
                    &state.world,
                    noun_map,
                ),
                |parse, container| parse.done_or_err(|| search(container, state, noun_map)),
                |input| Err(format!("\"{input}\" is not a valid searchable container.")),
            )
        }),
        parse.literal("give", |parse| {
            parse.match_against(
                super::crew_character_targets(&state.world, noun_map),
                |parse, receiver| {
                    parse.match_against(
                        inventory_items(state.controlled, &state.world, noun_map)
                            .into_iter()
                            .chain(items_in_hand(state.controlled, &state.world, noun_map)),
                        |parse, item| parse.done_or_err(|| give(receiver, item, state, noun_map)),
                        |input| {
                            Err(format!(
                                "{} has no {input} to give.",
                                NameData::find(&state.world, state.controlled, noun_map).definite(),
                            ))
                        },
                    )
                },
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
        parse.literal("wield", |parse| {
            parse.match_against(
                items_in_hand(state.controlled, &state.world, noun_map)
                    .into_iter()
                    .map(|(name, item)| (name, WieldItemTarget::InHand(item)))
                    .chain(
                        inventory_items(state.controlled, &state.world, noun_map)
                            .into_iter()
                            .map(|(name, item)| (name, WieldItemTarget::InInventory(item))),
                    )
                    .chain(
                        super::targets_by_proximity::<(&CanWield, &ItemType)>(
                            character_pos,
                            &state.world,
                            noun_map,
                        )
                        .into_iter()
                        .map(|(name, item)| (name, WieldItemTarget::OnGround(item))),
                    ),
                |parse, item| parse.done_or_err(|| wield(item, state, noun_map)),
                |input| {
                    Err(format!(
                        "There is no {input} that {} can wield.",
                        NameData::find(&state.world, state.controlled, noun_map).definite(),
                    ))
                },
            )
        }),
        parse.literal("use", |parse| {
            parse.match_against(
                items_in_hand(state.controlled, &state.world, noun_map)
                    .into_iter()
                    .chain(inventory_items(state.controlled, &state.world, noun_map)),
                |parse, item| parse.done_or_err(|| use_item(item, state, noun_map)),
                |input| Err(format!("No held item by the name \"{input}\".")),
            )
        }),
    )
}

fn inventory_items(
    character: Entity,
    world: &World,
    noun_map: &NounDataMap,
) -> Vec<(String, Entity)> {
    world
        .query::<&Held>()
        .with::<&ItemType>()
        .iter()
        .filter(|&(_, held)| held.is_in_inventory(character))
        .flat_map(|(entity, _)| {
            command::entity_names(world.entity(entity).unwrap(), noun_map)
                .into_iter()
                .map(move |name| (name, entity))
        })
        .collect()
}

fn items_in_hand(
    character: Entity,
    world: &World,
    noun_map: &NounDataMap,
) -> Vec<(String, Entity)> {
    world
        .query::<&Held>()
        .with::<&ItemType>()
        .iter()
        .filter(|&(_, held)| held.held_by(character) && held.is_in_hand())
        .flat_map(|(entity, _)| {
            command::entity_names(world.entity(entity).unwrap(), noun_map)
                .into_iter()
                .map(move |name| (name, entity))
        })
        .collect()
}

fn take_all(state: &GameState) -> Result<CommandResult, String> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    if !state
        .world
        .query::<&Pos>()
        .with::<&ItemType>()
        .iter()
        .any(|(_, pos)| pos.is_in(character_pos.get_area()))
    {
        return Err("There are no items to take here.".to_string());
    }

    if !behavior::is_safe(&state.world, character_pos.get_area()) {
        return Err("You should take care of all foes here before taking all items.".to_string());
    }

    command::action_result(Action::TakeAll)
}

fn take(item: Entity, state: &GameState, noun_map: &NounDataMap) -> Result<CommandResult, String> {
    super::check_accessible_with_message(item, state.controlled, true, &state.world, noun_map)?;

    command::action_result(Action::TakeItem(
        item,
        NameData::find(&state.world, item, noun_map),
    ))
}

fn search(
    container: Entity,
    state: &GameState,
    noun_map: &NounDataMap,
) -> Result<CommandResult, String> {
    super::check_accessible_with_message(
        container,
        state.controlled,
        true,
        &state.world,
        noun_map,
    )?;

    command::action_result(SearchAction { container })
}

fn give(
    receiver: Entity,
    item: Entity,
    state: &GameState,
    noun_map: &NounDataMap,
) -> Result<CommandResult, String> {
    if state.controlled == receiver {
        return Err(format!(
            "{} can't give an item to themselves.",
            NameData::find(&state.world, state.controlled, noun_map).definite()
        ));
    }

    super::check_adjacent_accessible_with_message(
        receiver,
        state.controlled,
        &state.world,
        noun_map,
    )?;

    command::action_result(Action::GiveItem(item, receiver))
}

enum WieldItemTarget {
    InHand(Entity),
    InInventory(Entity),
    OnGround(Entity),
}

fn wield(
    item: WieldItemTarget,
    state: &GameState,
    noun_map: &NounDataMap,
) -> Result<CommandResult, String> {
    match item {
        WieldItemTarget::InHand(item) => Err(format!(
            "{} is already wielding a {}.",
            NameData::find(&state.world, state.controlled, noun_map).definite(),
            NameData::find(&state.world, item, noun_map).base(),
        )),
        WieldItemTarget::InInventory(item) => command::action_result(Action::Wield(
            item,
            NameData::find(&state.world, item, noun_map),
        )),
        WieldItemTarget::OnGround(item) => {
            super::check_accessible_with_message(
                item,
                state.controlled,
                true,
                &state.world,
                noun_map,
            )?;

            command::action_result(Action::Wield(
                item,
                NameData::find(&state.world, item, noun_map),
            ))
        }
    }
}

fn use_item(
    item: Entity,
    state: &GameState,
    noun_map: &NounDataMap,
) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let item_ref = world.entity(item).unwrap();
    let item_type = *item_ref.get::<&ItemType>().unwrap();

    if item_type == ItemType::FuelCan {
        super::refuel_ship(state, noun_map)
    } else if item_type == ItemType::Medkit {
        if !world.get::<&Health>(character).unwrap().is_hurt() {
            return Err(format!(
                "{} is not hurt, and does not need to use the medkit.",
                NameData::find(world, character, noun_map).definite(),
            ));
        }
        command::action_result(UseAction { item })
    } else if item_type.is_usable() {
        command::action_result(UseAction { item })
    } else if item_ref.satisfies::<&CanWield>() {
        if item_ref
            .get::<&Held>()
            .is_some_and(|held| held.is_in_hand())
        {
            Err(format!(
                "{} is already being held.",
                NameData::find_by_ref(item_ref, noun_map).definite(),
            ))
        } else {
            command::action_result(Action::Wield(
                item,
                NameData::find_by_ref(item_ref, noun_map),
            ))
        }
    } else {
        Err("The item can not be used in any meaningful way.".to_string())
    }
}
