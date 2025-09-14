mod game;
mod parse;
mod store;
pub mod suggestion;

use crate::action::Action;
use crate::asset::GameAssets;
use crate::core;
use crate::core::name::NameData;
use crate::core::status::CreatureAttribute;
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

pub fn try_parse_input(
    input: &str,
    state: &GameState,
    assets: &GameAssets,
) -> Result<CommandResult, String> {
    if let Some(shopkeeper) = core::store::get_shop_info(&state.world, state.controlled) {
        store::parse(input, shopkeeper.deref(), state, assets)
    } else {
        game::parse(input, state, assets)
    }
    .map_err(text::capitalize)
}

fn status(state: &GameState, assets: &GameAssets) -> Result<CommandResult, String> {
    Ok(CommandResult::Info(CommandInfo::Status(
        view::get_full_status(state, assets),
    )))
}

fn entity_names(entity_ref: EntityRef<'_>, assets: &GameAssets) -> Vec<String> {
    match NameData::find_by_ref(entity_ref, assets) {
        NameData::Name(name) => vec![name],
        NameData::Noun(adjective, noun) => {
            let name = noun.singular();
            match (adjective, entity_ref.get::<&CreatureAttribute>().as_deref()) {
                (None, None) => vec![name.to_owned()],
                (None, Some(attribute)) => vec![name.to_owned(), format!("{attribute} {name}")],
                (Some(adjective), None) => vec![name.to_owned(), format!("{adjective} {name}")],
                (Some(adjective), Some(attribute)) => vec![
                    name.to_owned(),
                    format!("{attribute} {name}"),
                    format!("{adjective} {name}"),
                    format!("{adjective} {attribute} {name}"),
                ],
            }
        }
    }
}
