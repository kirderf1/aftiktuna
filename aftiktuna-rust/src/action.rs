use crate::{
    area::{Pos, Position},
    view::Messages,
};
use hecs::{Entity, World};
use Action::*;

pub mod door;
pub mod item;

pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    EnterDoor(Entity),
    ForceDoor(Entity),
}

fn try_move_aftik(world: &mut World, aftik: Entity, pos: Pos) -> Result<(), String> {
    let mut position = world.get_mut::<Position>(aftik).unwrap();
    assert_eq!(
        pos.get_area(),
        position.get_area(),
        "Areas should be equal when called."
    );

    position.0 = pos;
    Ok(())
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
