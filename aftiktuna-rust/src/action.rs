use crate::view::Messages;
use hecs::{Entity, World};
use Action::*;

pub mod combat;
pub mod door;
pub mod item;

#[derive(Debug, Default)]
pub struct Aftik;

pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Entity),
}

pub fn run_action(
    world: &mut World,
    performer: Entity,
    action: Action,
    controlled: Entity,
    messages: &mut Messages,
) {
    let result = match action {
        TakeItem(item, name) => item::take_item(world, performer, item, &name),
        TakeAll => item::take_all(world, performer),
        EnterDoor(door) => door::enter_door(world, performer, door),
        ForceDoor(door) => door::force_door(world, performer, door),
        Attack(target) => combat::attack(world, performer, target),
    };
    match result {
        Ok(message) => messages.0.push(message),
        Err(message) => {
            if performer == controlled {
                messages.0.push(message)
            }
        }
    }
}
