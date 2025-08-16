use crate::action::Action;
use crate::ai;
use crate::command::parse::{Parse, first_match_or};
use crate::command::{self, CommandResult};
use crate::core::position::Pos;
use crate::core::{Hostile, status};
use hecs::{Entity, EntityRef, World};
use std::collections::HashMap;

pub fn commands(
    parse: &Parse,
    performer_ref: EntityRef,
    world: &World,
) -> Option<Result<CommandResult, String>> {
    parse.literal("attack", |parse| {
        first_match_or!(
            parse.empty(|| attack_any(performer_ref, world));
            parse.match_against(
                hostile_targets(world, performer_ref.entity()),
                |parse, targets| parse.done_or_err(|| attack(performer_ref, targets, world)),
                |_| Err("There is no such target here.".to_string())
            )
        )
    })
}

fn attack_any(performer_ref: EntityRef, world: &World) -> Result<CommandResult, String> {
    let area = performer_ref.get::<&Pos>().unwrap().get_area();
    let foes = world
        .query::<&Pos>()
        .with::<&Hostile>()
        .iter()
        .filter(|&(entity, pos)| pos.is_in(area) && status::is_alive(entity, world))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    if foes.is_empty() {
        Err("There is no appropriate target to attack here.".to_string())
    } else {
        command::action_result(Action::Attack(
            foes,
            ai::pick_attack_kind(performer_ref, world, &mut rand::rng()),
        ))
    }
}

pub fn hostile_targets(world: &World, character: Entity) -> HashMap<String, Vec<Entity>> {
    let pos = *world.get::<&Pos>(character).unwrap();
    let mut map: HashMap<String, Vec<Entity>> = HashMap::new();
    world
        .query::<&Pos>()
        .with::<&Hostile>()
        .iter()
        .filter(|&(entity, target_pos)| {
            target_pos.is_in(pos.get_area()) && status::is_alive(entity, world)
        })
        .for_each(|(entity, _)| {
            for name in command::entity_names(world.entity(entity).unwrap()) {
                map.entry(name).or_default().push(entity);
            }
        });
    map
}

fn attack(
    performer_ref: EntityRef,
    targets: Vec<Entity>,
    world: &World,
) -> Result<CommandResult, String> {
    let character_pos = *performer_ref.get::<&Pos>().unwrap();

    let target_access = targets
        .iter()
        .map(|&entity| {
            (
                super::check_adjacent_accessible_with_message(
                    entity,
                    performer_ref.entity(),
                    world,
                ),
                *world.get::<&Pos>(entity).unwrap(),
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

    command::action_result(Action::Attack(
        targets,
        ai::pick_attack_kind(performer_ref, world, &mut rand::rng()),
    ))
}
