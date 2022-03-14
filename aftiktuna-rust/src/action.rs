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

    let min = min(aftik_pos.get_coord() + 1, pos.get_coord());
    let max = max(aftik_pos.get_coord() - 1, pos.get_coord());
    if world
        .query::<(&Position, &MovementBlocking)>()
        .iter()
        .any(|(_, (pos, _))| {
            pos.get_area() == aftik_pos.get_area()
                && min <= pos.get_coord()
                && pos.get_coord() <= max
        })
    {
        Err("Something is in the way.".to_string())
    } else {
        world.get_mut::<Position>(aftik).unwrap().0 = pos;
        Ok(())
    }
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
