use crate::action::Action;
use crate::asset::NounDataMap;
use crate::command;
use crate::command::CommandResult;
use crate::command::parse::{Parse, first_match_or};
use crate::core::inventory::Held;
use crate::core::name::{NameData, NameQuery};
use crate::core::store::{Shopkeeper, StoreStock};
use crate::game_loop::GameState;
use hecs::{Entity, World};
use std::collections::HashMap;

pub fn parse(
    input: &str,
    shopkeeper: &Shopkeeper,
    state: &GameState,
    noun_map: &NounDataMap,
) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let parse = Parse::new(input);
    first_match_or!(
        parse.literal("buy", |parse| {
            first_match_or!(
                parse.numeric(|parse, amount| {
                    parse.match_against(
                        store_entries(shopkeeper, amount, noun_map),
                        |parse, item| parse.done_or_err(|| buy(item, amount, noun_map)),
                        |input| Err(format!("\"{input}\" does not match an item in the store.")),
                    )
                });
                parse.match_against(
                    store_entries(shopkeeper, 1, noun_map),
                    |parse, item| parse.done_or_err(|| buy(item, 1, noun_map)),
                    |input| Err(format!("\"{input}\" does not match an item in the store.")),
                )
            )
        }),
        parse.literal("sell", |parse| {
            first_match_or!(
                parse.literal("all", |parse| {
                    parse.match_against(
                        held_item_lists_by_plurality(character, true, world, noun_map),
                        |parse, items| parse.done_or_err(|| command::action_result(Action::Sell(items))),
                        |input| Err(format!(
                            "{} is holding no item by the name \"{input}\".",
                            NameData::find(world, character, noun_map).definite(),
                        )),
                    )
                }),
                parse.numeric(|parse, count| {
                    parse.match_against(
                        held_item_lists_by_plurality(character, count != 1, world, noun_map),
                        |parse, items| parse.done_or_err(|| {
                            sell_count(count, prioritize_inventory(items, world), character, world, noun_map)
                        }),
                        |input| Err(format!(
                            "{} is holding no item by the name \"{input}\".",
                            NameData::find(world, character, noun_map).definite(),
                        )),
                    )
                });
                parse.match_against(
                    held_items(world, character, noun_map),
                    |parse, item| parse.done_or_err(|| command::action_result(Action::Sell(vec![item]))),
                    |input| {
                        Err(format!(
                            "\"{input}\" does not match an item in your inventory.",
                        ))
                    },
                )
            )
        }),
        parse.literal("exit", |parse| {
            parse.done_or_err(|| command::action_result(Action::ExitTrade))
        }),
        parse.literal("status", |parse| {
            parse.done_or_err(|| command::status(state, noun_map))
        });
        parse.default_err()
    )
}

fn store_entries<'a>(
    shopkeeper: &'a Shopkeeper,
    amount: u16,
    noun_map: &NounDataMap,
) -> Vec<(String, &'a StoreStock)> {
    shopkeeper
        .0
        .iter()
        .map(|stock| {
            (
                noun_map
                    .lookup(&stock.item.noun_id())
                    .for_count(amount)
                    .to_string(),
                stock,
            )
        })
        .collect::<Vec<_>>()
}

fn buy(stock: &StoreStock, amount: u16, noun_map: &NounDataMap) -> Result<CommandResult, String> {
    if stock.quantity.subtracted(amount).is_none() {
        return Err(format!(
            "There are not enough {} in stock.",
            noun_map.lookup(&stock.item.noun_id()).plural()
        ));
    }

    command::action_result(Action::Buy(stock.item, amount))
}

fn held_items(world: &World, character: Entity, noun_map: &NounDataMap) -> Vec<(String, Entity)> {
    let mut items = world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|(_, (_, held))| held.held_by(character))
        .map(|(entity, (query, held))| {
            (
                NameData::from_query(query, noun_map).base(),
                entity,
                held.is_in_hand(),
            )
        })
        .collect::<Vec<_>>();
    // Put item in hand at the end of the vec
    items.sort_by_key(|(_, _, in_hand)| *in_hand);
    items
        .into_iter()
        .map(|(name, entity, _)| (name, entity))
        .collect()
}

fn held_item_lists_by_plurality(
    character: Entity,
    plural: bool,
    world: &World,
    noun_map: &NounDataMap,
) -> HashMap<String, Vec<Entity>> {
    let mut map: HashMap<String, Vec<Entity>> = HashMap::new();
    world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|&(_, (_, held))| held.held_by(character))
        .for_each(|(entity, (name_query, _))| {
            let name_data = NameData::from_query(name_query, noun_map);
            let name = if plural {
                name_data.plural()
            } else {
                name_data.base()
            };
            map.entry(name).or_default().push(entity);
        });

    map
}

fn prioritize_inventory(mut items: Vec<Entity>, world: &World) -> Vec<Entity> {
    items.sort_by_key(|&item| world.get::<&Held>(item).unwrap().is_in_hand());
    items
}

fn sell_count(
    count: u16,
    items: Vec<Entity>,
    character: Entity,
    world: &World,
    noun_map: &NounDataMap,
) -> Result<CommandResult, String> {
    let count = usize::from(count);
    if items.len() < count {
        return Err(format!(
            "{} does not have that many {}.",
            NameData::find(world, character, noun_map).definite(),
            NameData::find(world, *items.first().unwrap(), noun_map).plural(),
        ));
    }
    command::action_result(Action::Sell(items[0..count].to_owned()))
}
