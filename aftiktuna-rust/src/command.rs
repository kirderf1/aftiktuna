use crate::action::{trade, Action};
use crate::view;
use crate::view::Messages;
use hecs::{Entity, World};
use std::ops::Deref;

mod game;
mod parse;
mod store;

pub enum Target {
    Controlled,
    Crew,
}

pub enum CommandResult {
    Action(Action, Target),
    ChangeControlled(Entity),
    None,
}

fn action_result(action: Action) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action, Target::Controlled))
}

fn crew_action(action: Action) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action, Target::Crew))
}

pub fn try_parse_input(
    input: &str,
    world: &World,
    character: Entity,
) -> Result<CommandResult, String> {
    if let Some(shopkeeper) = trade::get_shop_info(world, character) {
        store::parse(input, world, character, shopkeeper.deref())
    } else {
        game::parse(input, world, character)
    }
}

fn status(world: &World, character: Entity) -> Result<CommandResult, String> {
    let mut messages = Messages::default();
    view::print_full_status(world, character, &mut messages);
    messages.print_lines();
    Ok(CommandResult::None)
}
