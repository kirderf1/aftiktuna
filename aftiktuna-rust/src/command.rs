use crate::action::{trade, Action};
use crate::game_loop::GameState;
use crate::view;
use crate::view::text::Messages;
use hecs::Entity;
use std::ops::Deref;

mod game;
mod parse;
mod store;
pub mod suggestion;

#[derive(Debug, Copy, Clone)]
pub enum Target {
    Controlled,
    Crew,
}

pub enum CommandResult {
    Action(Action, Target),
    ChangeControlled(Entity),
    Info(Vec<String>),
}

fn action_result(action: impl Into<Action>) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action.into(), Target::Controlled))
}

fn crew_action(action: impl Into<Action>) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action.into(), Target::Crew))
}

pub fn try_parse_input(input: &str, state: &GameState) -> Result<CommandResult, String> {
    let input = input.to_lowercase();
    if let Some(shopkeeper) = trade::get_shop_info(&state.world, state.controlled) {
        store::parse(&input, shopkeeper.deref(), state)
    } else {
        game::parse(&input, state)
    }
}

fn status(state: &GameState) -> Result<CommandResult, String> {
    let mut messages = Messages::default();
    view::print_full_status(&mut messages, state);
    Ok(CommandResult::Info(messages.into_text()))
}
