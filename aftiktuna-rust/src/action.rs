use crate::{
    area::{Pos, Position},
    view::Messages,
};
use hecs::{Entity, World};
use std::cmp::{max, min};
use Action::*;

pub mod door;
pub mod item;

pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    EnterDoor(Entity),
    ForceDoor(Entity),
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

    let min = min(aftik_pos.get_coord() + 1, target_pos.get_coord());
    let max = if aftik_pos.get_coord() != 0 {
        max(aftik_pos.get_coord() - 1, target_pos.get_coord())
    } else {
        target_pos.get_coord()
    };
    world
        .query::<(&Position, &MovementBlocking)>()
        .iter()
        .any(|(_, (pos, _))| {
            pos.is_in(aftik_pos.get_area()) && min <= pos.get_coord() && pos.get_coord() <= max
        })
}

pub fn run_action(world: &mut World, aftik: Entity, messages: &mut Messages) {
    if let Ok(action) = world.remove_one::<Action>(aftik) {
        let result = match action {
            TakeItem(item, name) => item::take_item(item, &name, world, aftik),
            TakeAll => item::take_all(world, aftik),
            EnterDoor(door) => door::enter_door(door, world, aftik),
            ForceDoor(door) => door::force_door(door, world, aftik),
        };
        match result {
            Ok(message) | Err(message) => messages.0.push(message),
        }
    }
}
