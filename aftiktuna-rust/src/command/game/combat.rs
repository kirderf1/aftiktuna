use std::collections::HashMap;

use hecs::Entity;

use crate::action::Action;
use crate::command;
use crate::command::parse::{first_match_or, Parse};
use crate::command::CommandResult;
use crate::core::name::NameData;
use crate::core::position::Pos;
use crate::core::{status, CreatureAttribute, Hostile};
use crate::game_loop::GameState;

pub fn commands(parse: &Parse, state: &GameState) -> Option<Result<CommandResult, String>> {
    parse.literal("attack", |parse| {
        first_match_or!(
            parse.empty(|| attack_any(state));
            parse.match_against(
                get_targets_by_name(state),
                |parse, targets| parse.done_or_err(|| attack(targets, state)),
                |_| Err("There is no such target here.".to_string())
            )
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
        .with::<&Hostile>()
        .iter()
        .filter(|&(entity, pos)| pos.is_in(area) && status::is_alive(entity, &state.world))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    if foes.is_empty() {
        Err("There is no appropriate target to attack here.".to_string())
    } else {
        command::action_result(Action::Attack(foes))
    }
}

fn get_targets_by_name(state: &GameState) -> HashMap<String, Vec<Entity>> {
    let pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    let mut map: HashMap<String, Vec<Entity>> = HashMap::new();
    state
        .world
        .query::<&Pos>()
        .with::<&Hostile>()
        .iter()
        .filter(|&(entity, target_pos)| {
            target_pos.is_in(pos.get_area()) && status::is_alive(entity, &state.world)
        })
        .for_each(|(entity, _)| {
            let entity_ref = state.world.entity(entity).unwrap();
            let name_data = NameData::find_by_ref(entity_ref);
            map.entry(name_data.base().to_owned())
                .or_default()
                .push(entity);
            if let Some(attribute) = entity_ref.get::<&CreatureAttribute>() {
                map.entry(format!("{} {}", attribute.as_adjective(), name_data.base()))
                    .or_default()
                    .push(entity);
            }
        });
    map
}

fn attack(targets: Vec<Entity>, state: &GameState) -> Result<CommandResult, String> {
    let character_pos = *state.world.get::<&Pos>(state.controlled).unwrap();

    let target_access = targets
        .iter()
        .map(|&entity| {
            (
                super::check_adjacent_accessible_with_message(
                    &state.world,
                    state.controlled,
                    entity,
                ),
                *state.world.get::<&Pos>(entity).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    if target_access.iter().all(|(result, _)| result.is_err()) {
        return Err(target_access
            .into_iter()
            .min_by_key(|&(_, target_pos)| character_pos.distance_to(target_pos))
            .unwrap()
            .0
            .unwrap_err());
    }

    command::action_result(Action::Attack(targets))
}
