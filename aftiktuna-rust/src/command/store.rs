use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::Action;
use crate::command;
use crate::command::parse::Parse;
use crate::command::CommandResult;
use hecs::{Entity, Ref, World};

pub fn parse(
    input: &str,
    world: &World,
    character: Entity,
    shopkeeper: Ref<Shopkeeper>,
) -> Result<CommandResult, String> {
    Parse::new(input)
        .literal("buy", |parse| {
            parse.match_against(
                store_entries(shopkeeper),
                |parse, item| parse.done_or_err(|| buy(item)),
                |input| {
                    Err(format!(
                        "\"{}\" does not match an item in the store.",
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

fn store_entries(shopkeeper: Ref<Shopkeeper>) -> Vec<(String, PricedItem)> {
    shopkeeper
        .0
        .iter()
        .map(|priced| {
            (
                priced.item.display_info().name().to_string(),
                priced.clone(),
            )
        })
        .collect::<Vec<_>>()
}

fn buy(priced_item: PricedItem) -> Result<CommandResult, String> {
    command::action_result(Action::Buy(priced_item.item))
}
