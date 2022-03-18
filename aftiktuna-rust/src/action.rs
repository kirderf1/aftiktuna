use crate::{
    area::{Pos, Position},
    view::Messages,
};
use hecs::{Entity, With, World};
use std::cmp::{max, min};
use Action::*;

pub mod combat;
pub mod door;
pub mod item;

pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Entity),
}

#[derive(Debug, Default)]
pub struct MovementBlocking;

fn try_move_aftik(world: &mut World, aftik: Entity, pos: Pos) -> Result<(), String> {
    let aftik_pos = world.get::<Position>(aftik).unwrap().0;
    assert_eq!(
        pos.get_area(),
        aftik_pos.get_area(),
        "Areas should be equal when called."
    );

    if is_blocked_for_aftik(world, aftik_pos, pos) {
        Err("Something is in the way.".to_string())
    } else {
        world.get_mut::<Position>(aftik).unwrap().0 = pos;
        Ok(())
    }
}

pub fn is_blocked_for_aftik(world: &World, aftik_pos: Pos, target_pos: Pos) -> bool {
    if aftik_pos.get_coord() == target_pos.get_coord() {
        return false;
    }

    let adjacent_pos = aftik_pos.get_adjacent_towards(target_pos);
    let min = min(adjacent_pos.get_coord(), target_pos.get_coord());
    let max = max(adjacent_pos.get_coord(), target_pos.get_coord());
    world
        .query::<With<MovementBlocking, &Position>>()
        .iter()
        .any(|(_, pos)| {
            pos.is_in(aftik_pos.get_area()) && min <= pos.get_coord() && pos.get_coord() <= max
        })
}

pub fn run_action(world: &mut World, aftik: Entity, messages: &mut Messages) {
    if let Ok(action) = world.remove_one::<Action>(aftik) {
        let result = match action {
            TakeItem(item, name) => item::take_item(world, aftik, item, &name),
            TakeAll => item::take_all(world, aftik),
            EnterDoor(door) => door::enter_door(world, aftik, door),
            ForceDoor(door) => door::force_door(world, aftik, door),
            Attack(target) => combat::attack(world, aftik, target),
        };
        match result {
            Ok(message) | Err(message) => messages.0.push(message),
        }
    }
}
