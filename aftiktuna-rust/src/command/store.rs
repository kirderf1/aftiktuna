use crate::action::Action;
use crate::command;
use crate::command::parse::Parse;
use crate::command::CommandResult;
use hecs::{Entity, World};

pub fn parse(input: &str, world: &World, character: Entity) -> Result<CommandResult, String> {
    Parse::new(input)
        .literal("buy", |parse| {
            parse.done_or_err(|| command::action_result(Action::Buy))
        })
        .literal("exit", |parse| {
            parse.done_or_err(|| command::action_result(Action::ExitTrade))
        })
        .literal("status", |parse| {
            parse.done_or_err(|| command::status(world, character))
        })
        .or_else_err(|| format!("Unexpected input: \"{}\"", input))
}
