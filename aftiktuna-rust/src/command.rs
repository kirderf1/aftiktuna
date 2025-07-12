mod game;
mod parse;
mod store;
pub mod suggestion;

use crate::action::{Action, trade};
use crate::core::CreatureAttribute;
use crate::core::name::NameData;
use crate::game_loop::GameState;
use crate::view::{self, text};
use hecs::{Entity, EntityRef};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub enum Target {
    Controlled,
    Crew,
}

pub enum CommandResult {
    Action(Action, Target),
    ChangeControlled(Entity),
    Info(CommandInfo),
}

pub enum CommandInfo {
    Message(Vec<String>),
    Status(view::FullStatus),
}

impl CommandInfo {
    pub fn into_text(self) -> Vec<String> {
        match self {
            CommandInfo::Message(text_lines) => text_lines,
            CommandInfo::Status(status) => status.into_text(),
        }
    }
}

impl From<Vec<String>> for CommandInfo {
    fn from(value: Vec<String>) -> Self {
        Self::Message(value)
    }
}

fn action_result(action: impl Into<Action>) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action.into(), Target::Controlled))
}

fn crew_action(action: impl Into<Action>) -> Result<CommandResult, String> {
    Ok(CommandResult::Action(action.into(), Target::Crew))
}

pub fn try_parse_input(input: &str, state: &GameState) -> Result<CommandResult, String> {
    if let Some(shopkeeper) = trade::get_shop_info(&state.world, state.controlled) {
        store::parse(input, shopkeeper.deref(), state)
    } else {
        game::parse(input, state)
    }
    .map_err(text::capitalize)
}

fn status(state: &GameState) -> Result<CommandResult, String> {
    Ok(CommandResult::Info(CommandInfo::Status(
        view::get_full_status(state),
    )))
}

fn entity_names(entity_ref: EntityRef<'_>) -> Vec<String> {
    let name_data = NameData::find_by_ref(entity_ref);
    let name = name_data.base().to_lowercase();
    if let NameData::Noun(noun) = name_data
        && let Some(attribute) = entity_ref.get::<&CreatureAttribute>()
    {
        vec![
            name,
            format!("{} {}", attribute.as_adjective(), noun.singular()),
        ]
    } else {
        vec![name]
    }
}
