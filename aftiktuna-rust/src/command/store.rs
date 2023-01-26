use crate::action::item::Held;
use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::Action;
use crate::command;
use crate::command::parse::Parse;
use crate::command::CommandResult;
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
            parse.match_against(
                inventory_items(world, character),
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

fn store_entries(shopkeeper: &Shopkeeper, amount: i32) -> Vec<(String, &PricedItem)> {
    shopkeeper
        .0
        .iter()
        .map(|priced| (priced.item.name_for_amount(amount), priced))
        .collect::<Vec<_>>()
}

fn buy(priced_item: &PricedItem, amount: i32) -> Result<CommandResult, String> {
    command::action_result(Action::Buy(priced_item.item, amount))
}

fn inventory_items(world: &World, character: Entity) -> Vec<(String, Entity)> {
    world
        .query::<(&NameData, &Held)>()
        .iter()
        .filter(|(_, (_, held))| held.held_by(character))
        .map(|(entity, (name, _))| (name.base().to_string(), entity))
        .collect::<Vec<_>>()
}

fn sell(item: Entity) -> Result<CommandResult, String> {
    command::action_result(Action::Sell(item))
}
