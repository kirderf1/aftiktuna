use crate::action::Action;
use crate::command;
use crate::command::parse::{first_match_or, Parse};
use crate::command::CommandResult;
use crate::core::position::Pos;
use crate::core::IsFoe;
use crate::game_loop::GameState;
use crate::view::name::{NameData, NameQuery};

pub fn commands(parse: &Parse, state: &GameState) -> Option<Result<CommandResult, String>> {
    parse.literal("attack", |parse| {
        first_match_or!(
            parse.empty(|| attack_any(state));
            parse.take_remaining(|target_name| attack(target_name, state))
        )
    })
}

fn attack_any(state: &GameState) -> Result<CommandResult, String> {
    let area = state
        .world
        .get::<&Pos>(state.controlled)
        .unwrap()
        .get_area();
    let foes = state
        .world
        .query::<&Pos>()
        .with::<&IsFoe>()
        .iter()
        .filter(|(_, pos)| pos.is_in(area))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    if foes.is_empty() {
        Err("There is no appropriate target to attack here.".to_string())
    } else {
        command::action_result(Action::Attack(foes))
    }
}

fn attack(target_name: &str, state: &GameState) -> Result<CommandResult, String> {
    let pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    let targets = state
        .world
        .query::<(&Pos, NameQuery)>()
        .with::<&IsFoe>()
        .iter()
        .filter(|&(_, (target_pos, query))| {
            target_pos.is_in(pos.get_area()) && NameData::from(query).matches(target_name)
        })
        .map(|(target, (&pos, _))| (target, pos))
        .collect::<Vec<_>>();

    if targets.is_empty() {
        return Err("There is no such target here.".to_string());
    }

    let target_access = targets
        .iter()
        .map(|&(entity, pos)| {
            (
                super::check_adjacent_accessible_with_message(
                    &state.world,
                    state.controlled,
                    entity,
                ),
                pos,
            )
        })
        .collect::<Vec<_>>();
    if target_access.iter().all(|(result, _)| result.is_err()) {
        return Err(target_access
            .into_iter()
            .min_by_key(|&(_, target_pos)| pos.distance_to(target_pos))
            .unwrap()
            .0
            .unwrap_err());
    }

    command::action_result(Action::Attack(
        targets.into_iter().map(|(entity, _)| entity).collect(),
    ))
}
