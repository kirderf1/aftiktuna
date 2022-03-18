use crate::view::Messages;
use hecs::{Entity, World};
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
