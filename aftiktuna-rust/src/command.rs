use crate::action::{trade, Action};
use crate::view;
use hecs::{Entity, World};

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
    if trade::get_shop_info(world, character).is_some() {
        store::parse(input, world, character)
    } else {
        game::parse(input, world, character)
    }
}

fn status(world: &World, character: Entity) -> Result<CommandResult, String> {
    view::print_full_status(world, character);
    Ok(CommandResult::None)
}
