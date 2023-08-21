use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::Action;
use crate::command;
use crate::command::parse::Parse;
use crate::command::CommandResult;
use crate::core::inventory::Held;
use crate::view::NameData;
use hecs::{Entity, World};

pub fn parse(
    input: &str,
    world: &World,
    character: Entity,
    shopkeeper: &Shopkeeper,
) -> Result<CommandResult, String> {
    Parse::new(input)
        .literal("buy", |parse| {
            parse
                .numeric(|parse, amount| {
                    parse.match_against(
                        store_entries(shopkeeper, amount),
                        |parse, item| parse.done_or_err(|| buy(item, amount)),
                        |input| {
                            Err(format!(
                                "\"{}\" does not match an item in the store.",
                                input
                            ))
                        },
                    )
                })
                .match_against(
                    store_entries(shopkeeper, 1),
                    |parse, item| parse.done_or_err(|| buy(item, 1)),
                    |input| {
                        Err(format!(
                            "\"{}\" does not match an item in the store.",
                            input
                        ))
                    },
                )
        })
        .literal("sell", |parse| {
            parse
                .numeric(|parse, count| {
                    parse.take_remaining(|item_name| sell_count(world, character, count, item_name))
                })
                .literal("all", |parse| {
                    parse.take_remaining(|item_name| sell_all(world, character, item_name))
                })
                .match_against(
                    held_items(world, character),
                    |parse, item| parse.done_or_err(|| sell(item)),
                    |input| {
                        Err(format!(
                            "\"{}\" does not match an item in your inventory.",
                            input
                        ))
                    },
                )
        })
        .literal("exit", |parse| {
            parse.done_or_err(|| command::action_result(Action::ExitTrade))
        })
        .literal("status", |parse| {
            parse.done_or_err(|| command::status(world, character))
        })
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}

fn store_entries(shopkeeper: &Shopkeeper, amount: u16) -> Vec<(String, &PricedItem)> {
    shopkeeper
        .0
        .iter()
        .map(|priced| {
            (
                priced.item.noun_data().for_count(amount).to_string(),
                priced,
            )
        })
        .collect::<Vec<_>>()
}

fn buy(priced_item: &PricedItem, amount: u16) -> Result<CommandResult, String> {
    command::action_result(Action::Buy(priced_item.item, amount))
}

fn held_items(world: &World, character: Entity) -> Vec<(String, Entity)> {
    let mut items = world
        .query::<(&NameData, &Held)>()
        .iter()
        .filter(|(_, (_, held))| held.held_by(character))
        .map(|(entity, (name, held))| (name.base().to_string(), entity, held.is_in_hand()))
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
        .query::<(&NameData, &Held)>()
        .iter()
        .filter(|(_, (name_data, held))| {
            name_data.matches_with_count(item_name, count) && held.held_by(character)
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
        .query::<(&NameData, &Held)>()
        .iter()
        .filter(|(_, (name_data, held))| {
            name_data.matches_plural(item_name) && held.held_by(character)
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
