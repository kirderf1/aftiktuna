use crate::action::Action;
use crate::command;
use crate::command::CommandResult;
use crate::command::parse::{Parse, first_match, first_match_or};
use crate::core::name::{Name, NameData, NameQuery};
use crate::core::position::Pos;
use crate::core::{Character, Recruitable, Waiting, area, status};
use crate::game_loop::GameState;
use hecs::Entity;

pub fn commands(parse: &Parse, state: &GameState) -> Option<Result<CommandResult, String>> {
    first_match!(
        parse.literal("talk", |parse| {
            first_match_or!(
                parse.literal("to", |parse| {
                    parse.match_against(
                        talk_targets(state),
                        |parse, target| parse.done_or_err(|| talk_to(state, target)),
                        |input| Err(format!("\"{input}\" is not a valid target.")),
                    )
                });
                parse.default_err()
            )
        }),
        parse.literal("recruit", |parse| {
            parse.match_against(
                recruit_targets(state),
                |parse, target| parse.done_or_err(|| recruit(state, target)),
                |input| Err(format!("\"{input}\" is not a valid recruitment target.")),
            )
        }),
        parse.literal("tell", |parse| {
            parse.match_against(
                super::crew_character_targets(&state.world),
                |parse, target| {
                    first_match_or!(
                        parse.literal("to", |parse|
                            subcommands_for_tell(parse, state, target));
                        parse.default_err()
                    )
                },
                |input| Err(format!("\"{input}\" is not a valid target.")),
            )
        }),
    )
}

fn subcommands_for_tell(
    parse: Parse,
    state: &GameState,
    target: Entity,
) -> Result<CommandResult, String> {
    first_match_or!(
        parse.literal("wait", |parse|
            first_match_or!(
                parse.literal("at ship", |parse|
                    parse.done_or_err(|| tell_to_wait_at_ship(state, target)));
                parse.done_or_err(|| tell_to_wait(state, target))
            )
        ),
        parse.literal("follow", |parse|
            parse.done_or_err(|| tell_to_follow(state, target)));
        parse.default_err()
    )
}

fn talk_targets(state: &GameState) -> Vec<(String, Entity)> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    state
        .world
        .query::<(NameQuery, &Pos)>()
        .with::<(&Name, &Character)>()
        .iter()
        .filter(|(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (query, _))| (NameData::from(query).base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

fn talk_to(state: &GameState, target: Entity) -> Result<CommandResult, String> {
    if target == state.controlled {
        return Err(format!(
            "{} does not want to talk to themselves.",
            NameData::find(&state.world, state.controlled).definite()
        ));
    }
    if !status::is_alive(target, &state.world) {
        return Err(format!(
            "{} cannot talk to the dead.",
            NameData::find(&state.world, state.controlled).definite()
        ));
    }

    super::check_adjacent_accessible_with_message(target, state.controlled, &state.world)?;

    command::action_result(Action::TalkTo(target))
}

fn recruit_targets(state: &GameState) -> Vec<(String, Entity)> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    state
        .world
        .query::<(NameQuery, &Pos)>()
        .with::<(&Recruitable, &Character)>()
        .iter()
        .filter(|(_, (_, pos))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (query, _))| (NameData::from(query).base().to_lowercase(), entity))
        .collect::<Vec<_>>()
}

fn recruit(state: &GameState, target: Entity) -> Result<CommandResult, String> {
    super::check_adjacent_accessible_with_message(target, state.controlled, &state.world)?;

    command::action_result(Action::Recruit(target))
}

fn tell_to_wait(state: &GameState, target: Entity) -> Result<CommandResult, String> {
    if state.controlled == target {
        return Err(format!(
            "{} can't give an order to themselves.",
            NameData::find(&state.world, state.controlled).definite()
        ));
    }
    let controlled_pos = state.world.get::<&Pos>(state.controlled).unwrap();
    let target_pos = state.world.get::<&Pos>(target).unwrap();
    if !controlled_pos.is_in(target_pos.get_area()) {
        return Err(format!(
            "{} can't tell {} to do things from here.",
            NameData::find(&state.world, state.controlled).definite(),
            NameData::find(&state.world, target).definite()
        ));
    }

    if state.world.satisfies::<&Waiting>(target).unwrap_or(false) {
        return Err(format!(
            "{} is already waiting.",
            NameData::find(&state.world, target).definite()
        ));
    }

    command::action_result(Action::TellToWait(target))
}

fn tell_to_wait_at_ship(state: &GameState, target: Entity) -> Result<CommandResult, String> {
    if state.controlled == target {
        return Err(format!(
            "{} can't give an order to themselves.",
            NameData::find(&state.world, state.controlled).definite()
        ));
    }
    let controlled_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    let target_pos = *state.world.get::<&Pos>(target).unwrap();
    if !controlled_pos.is_in(target_pos.get_area()) {
        return Err(format!(
            "{} can't tell {} to do things from here.",
            NameData::find(&state.world, state.controlled).definite(),
            NameData::find(&state.world, target).definite()
        ));
    }
    if area::is_in_ship(target_pos, &state.world)
        && state.world.satisfies::<&Waiting>(target).unwrap_or(false)
    {
        return Err(format!(
            "{} is already waiting at the ship.",
            NameData::find(&state.world, target).definite()
        ));
    }
    if state
        .world
        .get::<&Waiting>(target)
        .is_ok_and(|waiting| waiting.at_ship)
    {
        return Err(format!(
            "{} is already on their way to the ship.",
            NameData::find(&state.world, target).definite()
        ));
    }

    command::action_result(Action::TellToWaitAtShip(target))
}

fn tell_to_follow(state: &GameState, target: Entity) -> Result<CommandResult, String> {
    if state.controlled == target {
        return Err(format!(
            "{} can't give an order to themselves.",
            NameData::find(&state.world, state.controlled).definite()
        ));
    }
    let controlled_pos = state.world.get::<&Pos>(state.controlled).unwrap();
    let target_pos = state.world.get::<&Pos>(target).unwrap();
    if !controlled_pos.is_in(target_pos.get_area()) {
        return Err(format!(
            "{} can't tell {} to do things from here.",
            NameData::find(&state.world, state.controlled).definite(),
            NameData::find(&state.world, target).definite()
        ));
    }

    command::action_result(Action::TellToFollow(target))
}
