use crate::action::Action;
use crate::command;
use crate::command::CommandResult;
use crate::command::parse::{Parse, first_match_or};
use crate::core::inventory::Held;
use crate::core::name::{NameData, NameQuery};
use crate::core::store::{Shopkeeper, StoreStock};
use crate::game_loop::GameState;
use hecs::{Entity, World};

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
                    parse.take_remaining(|item_name| sell_all(world, character, item_name))
                }),
                parse.numeric(|parse, count| {
                    parse.take_remaining(|item_name| sell_count(world, character, count, item_name))
                });
                parse.match_against(
                    held_items(world, character),
                    |parse, item| parse.done_or_err(|| sell(item)),
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
        .map(|(entity, (query, held))| {
            (
                NameData::from(query).base().to_string(),
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

fn sell(item: Entity) -> Result<CommandResult, String> {
    command::action_result(Action::Sell(vec![item]))
}

fn sell_count(
    world: &World,
    character: Entity,
    count: u16,
    item_name: &str,
) -> Result<CommandResult, String> {
    let mut items = world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|&(_, (query, held))| {
            NameData::from(query).matches_with_count(item_name, count) && held.held_by(character)
        })
        .map(|(entity, (_, held))| (entity, held.is_in_hand()))
        .collect::<Vec<_>>();
    // Put item in hand at the end of the vec
    items.sort_by_key(|(_, in_hand)| *in_hand);

    if items.is_empty() {
        return Err(format!(
            "{} is holding no item by the name \"{}\".",
            NameData::find(world, character).definite(),
            item_name
        ));
    }
    let count = count as usize;
    if items.len() < count {
        return Err(format!(
            "{} does not have that many {}.",
            NameData::find(world, character).definite(),
            item_name
        ));
    }
    command::action_result(Action::Sell(
        items[0..count].iter().map(|(item, _)| *item).collect(),
    ))
}

fn sell_all(world: &World, character: Entity, item_name: &str) -> Result<CommandResult, String> {
    let items = world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|&(_, (query, held))| {
            NameData::from(query).matches_plural(item_name) && held.held_by(character)
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    if items.is_empty() {
        return Err(format!(
            "{} is holding no item by the name \"{}\".",
            NameData::find(world, character).definite(),
            item_name
        ));
    }
    command::action_result(Action::Sell(items))
}
