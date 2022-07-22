use crate::position::Pos;
use crate::status;
use crate::view::{DisplayInfo, Messages};
use hecs::{Entity, With, World};
use Action::*;

pub mod combat;
pub mod door;
pub mod item;
mod launch;

#[derive(Debug, Default)]
pub struct Aftik;

pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    Wield(Entity, String),
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Entity),
    Wait,
    Rest(bool),
    Launch,
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

pub fn perform(
    world: &mut World,
    performer: Entity,
    action: Action,
    controlled: Entity,
    messages: &mut Messages,
) {
    let result = match action {
        TakeItem(item, name) => item::take_item(world, performer, item, &name).map(Some),
        TakeAll => item::take_all(world, performer).map(Some),
        Wield(item, name) => item::wield(world, performer, item, &name).map(Some),
        EnterDoor(door) => door::enter_door(world, performer, door).map(Some),
        ForceDoor(door) => door::force_door(world, performer, door).map(Some),
        Attack(target) => combat::attack(world, performer, target).map(Some),
        Wait => Ok(None),
        Rest(first) => Ok(rest(world, performer, first)),
        Launch => Ok(launch::perform(world, performer)),
    };
    match result {
        Ok(Some(message)) => messages.add(message),
        Ok(None) => {}
        Err(message) => {
            if performer == controlled {
                messages.add(message);
            }
        }
    }
}

fn rest(world: &mut World, performer: Entity, first: bool) -> Option<String> {
    let need_more_rest = world
        .get::<status::Stamina>(performer)
        .map(|stamina| stamina.need_more_rest())
        .unwrap_or(false);

    if need_more_rest {
        world.insert_one(performer, Rest(false)).unwrap();
    }

    if first {
        Some(format!(
            "{} takes some time to rest up.",
            DisplayInfo::find_definite_name(world, performer)
        ))
    } else {
        None
    }
}
