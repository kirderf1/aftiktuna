use crate::action::Action;
use crate::asset::{GameAssets, NounDataMap};
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
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    let world = &state.world;
    let character = state.controlled;
    let parse = Parse::new(input);
    first_match_or!(
        parse.literal("buy", |parse| {
            first_match_or!(
                parse.numeric(|parse, amount| {
                    parse.match_against(
                        store_entries(shopkeeper, amount, &assets.noun_data_map),
                        |parse, item| parse.done_or_err(|| buy(item, amount, &assets.noun_data_map)),
                        |input| Err(format!("\"{input}\" does not match an item in the store.")),
                    )
                });
                parse.match_against(
                    store_entries(shopkeeper, 1, &assets.noun_data_map),
                    |parse, item| parse.done_or_err(|| buy(item, 1, &assets.noun_data_map)),
                    |input| Err(format!("\"{input}\" does not match an item in the store.")),
                )
            )
        }),
        parse.literal("sell", |parse| {
            first_match_or!(
                parse.literal("all", |parse| {
                    parse.match_against(
                        held_item_lists_by_plurality(character, true, world, assets),
                        |parse, items| parse.done_or_err(|| command::action_result(Action::Sell(items))),
                        |input| Err(format!(
                            "{} is holding no item by the name \"{input}\".",
                            NameData::find(world, character, assets).definite(),
                        )),
                    )
                }),
                parse.numeric(|parse, count| {
                    parse.match_against(
                        held_item_lists_by_plurality(character, count != 1, world, assets),
                        |parse, items| parse.done_or_err(|| {
                            sell_count(count, prioritize_inventory(items, world), character, world, assets)
                        }),
                        |input| Err(format!(
                            "{} is holding no item by the name \"{input}\".",
                            NameData::find(world, character, assets).definite(),
                        )),
                    )
                });
                parse.match_against(
                    held_items(world, character, assets),
                    |parse, item| parse.done_or_err(|| command::action_result(Action::Sell(vec![item]))),
                    |input| {
                        Err(format!(
                            "\"{input}\" does not match an item in your inventory.",
                        ))
                    },
                )
            )
        }),
        parse.literal("ask about", |parse| {
                    parse.match_against(
                        store_entries(shopkeeper, 1, &assets.noun_data_map),
                        |parse, stock| parse.done_or_err(|| command::action_result(Action::AskAbout(stock.item.clone()))),
                        |input| Err(format!("\"{input}\" does not match an item in the store.")),
                    )
        }),
        parse.literal("exit", |parse| {
            parse.done_or_err(|| command::action_result(Action::ExitTrade))
        }),
        parse.literal("status", |parse| {
            parse.done_or_err(|| command::status(state, assets))
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

    command::action_result(Action::Buy(stock.item.clone(), amount))
}

fn held_items(world: &World, character: Entity, assets: &GameAssets) -> Vec<(String, Entity)> {
    let mut items = world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|(_, (_, held))| held.held_by(character))
        .map(|(entity, (query, held))| {
            (
                NameData::from_query(query, assets).base(),
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
    assets: &GameAssets,
) -> HashMap<String, Vec<Entity>> {
    let mut map: HashMap<String, Vec<Entity>> = HashMap::new();
    world
        .query::<(NameQuery, &Held)>()
        .iter()
        .filter(|&(_, (_, held))| held.held_by(character))
        .for_each(|(entity, (name_query, _))| {
            let name_data = NameData::from_query(name_query, assets);
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
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    let count = usize::from(count);
    if items.len() < count {
        return Err(format!(
            "{} does not have that many {}.",
            NameData::find(world, character, assets).definite(),
            NameData::find(world, *items.first().unwrap(), assets).plural(),
        ));
    }
    command::action_result(Action::Sell(items[0..count].to_owned()))
}
