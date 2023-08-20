use crate::action::{trade, Action};
use crate::core::GameState;
use crate::view;
use crate::view::Messages;
use hecs::{Entity, World};
use std::ops::Deref;

mod game;
mod parse;
mod store;

#[derive(Copy, Clone)]
pub enum Target {
    Controlled,
    Crew,
}

pub enum CommandResult {
    Action(Action, Target),
    ChangeControlled(Entity),
    Info(Messages),
}

fn action_result(action: Action) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action, Target::Controlled))
}

fn crew_action(action: Action) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action, Target::Crew))
}

pub fn try_parse_input(input: &str, state: &GameState) -> Result<CommandResult, String> {
    let input = input.to_lowercase();
    if let Some(shopkeeper) = trade::get_shop_info(&state.world, state.controlled) {
        store::parse(&input, &state.world, state.controlled, shopkeeper.deref())
    } else {
        game::parse(&input, state)
    }
}

fn status(world: &World, character: Entity) -> Result<CommandResult, String> {
    let mut messages = Messages::default();
    view::print_full_status(world, character, &mut messages);
    Ok(CommandResult::Info(messages))
}
