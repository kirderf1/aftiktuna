use crate::position::Pos;
use crate::status;
use crate::view::Messages;
use hecs::{Entity, With, World};
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

pub fn foe_ai(world: &mut World, foe: Entity) {
    if status::is_alive(foe, world) && world.get::<Action>(foe).is_err() {
        if let Some(action) = pick_foe_action(world, foe) {
            world.insert_one(foe, action).unwrap();
        }
    }
}

fn pick_foe_action(world: &World, foe: Entity) -> Option<Action> {
    let pos = *world.get::<Pos>(foe).ok()?;
    let target = world
        .query::<With<Aftik, &Pos>>()
        .iter()
        .filter(|(_, aftik_pos)| aftik_pos.is_in(pos.get_area()))
        .min_by_key(|(_, aftik_pos)| aftik_pos.distance_to(pos))
        .map(|(aftik, _)| aftik);
    target.map(Attack)
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
