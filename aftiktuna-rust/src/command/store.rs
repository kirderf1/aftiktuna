use crate::action::Action;
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
) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let parse = Parse::new(input);
    first_match_or!(
        parse.literal("buy", |parse| {
            first_match_or!(
                parse.numeric(|parse, amount| {
                    parse.match_against(
                        store_entries(shopkeeper, amount),
                        |parse, item| parse.done_or_err(|| buy(item, amount)),
                        |input| Err(format!("\"{input}\" does not match an item in the store.")),
                    )
                });
                parse.match_against(
                    store_entries(shopkeeper, 1),
                    |parse, item| parse.done_or_err(|| buy(item, 1)),
                    |input| Err(format!("\"{input}\" does not match an item in the store.")),
                )
            )
        }),
        parse.literal("sell", |parse| {
            first_match_or!(
                parse.literal("all", |parse| {
                    parse.match_against(
                        held_item_lists_by_plurality(character, true, world),
                        |parse, items| parse.done_or_err(|| command::action_result(Action::Sell(items))),
                        |input| Err(format!(
                            "{} is holding no item by the name \"{input}\".",
                            NameData::find(world, character).definite(),
                        )),
                    )
                }),
                parse.numeric(|parse, count| {
                    parse.match_against(
                        held_item_lists_by_plurality(character, count != 1, world),
                        |parse, items| parse.done_or_err(|| {
                            sell_count(count, prioritize_inventory(items, world), character, world)
                        }),
                        |input| Err(format!(
                            "{} is holding no item by the name \"{input}\".",
                            NameData::find(world, character).definite(),
                        )),
                    )
                });
                parse.match_against(
                    held_items(world, character),
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
            parse.done_or_err(|| command::status(state))
        });
        parse.default_err()
    )
}

fn store_entries(shopkeeper: &Shopkeeper, amount: u16) -> Vec<(String, &StoreStock)> {
    shopkeeper
        .0
        .iter()
        .map(|stock| (stock.item.noun_data().for_count(amount).to_string(), stock))
        .collect::<Vec<_>>()
}

fn buy(stock: &StoreStock, amount: u16) -> Result<CommandResult, String> {
    if stock.quantity.subtracted(amount).is_none() {
        return Err(format!(
            "There are not enough {} in stock.",
            stock.item.noun_data().plural()
        ));
    }

    command::action_result(Action::Buy(stock.item, amount))
}

fn held_items(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let mut items = world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|(_, (_, held))| held.held_by(character))
        .map(|(entity, (query, held))| (NameData::from(query).base(), entity, held.is_in_hand()))
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
) -> HashMap<String, Vec<Entity>> {
    let mut map: HashMap<String, Vec<Entity>> = HashMap::new();
    world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|&(_, (_, held))| held.held_by(character))
        .for_each(|(entity, (name_query, _))| {
            let name_data = NameData::from(name_query);
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
) -> Result<CommandResult, String> {
    let count = usize::from(count);
    if items.len() < count {
        return Err(format!(
            "{} does not have that many {}.",
            NameData::find(world, character).definite(),
            NameData::find(world, *items.first().unwrap()).plural(),
        ));
    }
    command::action_result(Action::Sell(items[0..count].to_owned()))
}
