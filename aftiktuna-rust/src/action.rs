use crate::action::combat::{IsFoe, Target};
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

#[derive(Clone)]
pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    GiveItem(Entity, Entity),
    Wield(Entity, String),
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Entity),
    AttackNearest(Target),
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
    if world
        .query::<With<Aftik, &Pos>>()
        .iter()
        .any(|(_, aftik_pos)| aftik_pos.is_in(pos.get_area()))
    {
        Some(AttackNearest(Target::Aftik))
    } else {
        None
    }
}

pub fn aftik_ai(world: &mut World, aftik: Entity) {
    if status::is_alive(aftik, world) && world.get::<Action>(aftik).is_err() {
        if let Some(action) = pick_aftik_action(world, aftik) {
            world.insert_one(aftik, action).unwrap();
        }
    }
}

fn pick_aftik_action(world: &World, aftik: Entity) -> Option<Action> {
    let pos = *world.get::<Pos>(aftik).ok()?;
    if world
        .query::<With<IsFoe, &Pos>>()
        .iter()
        .any(|(_, foe_pos)| foe_pos.is_in(pos.get_area()))
    {
        Some(AttackNearest(Target::Foe))
    } else {
        None
    }
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
        GiveItem(item, receiver) => item::give_item(world, performer, item, receiver).map(Some),
        Wield(item, name) => item::wield(world, performer, item, &name).map(Some),
        EnterDoor(door) => door::enter_door(world, performer, door).map(Some),
        ForceDoor(door) => door::force_door(world, performer, door).map(Some),
        Attack(target) => combat::attack(world, performer, target).map(Some),
        AttackNearest(target) => combat::attack_nearest(world, performer, target),
        Wait => Ok(None),
        Rest(first) => Ok(rest(world, performer, first)),
        Launch => Ok(launch::perform(world, performer)),
    };
    match result {
        Ok(Some(message)) => {
            let performer_pos = *world.get::<Pos>(performer).unwrap();
            let player_pos = *world.get::<Pos>(controlled).unwrap();
            if player_pos.is_in(performer_pos.get_area()) {
                messages.add(message);
            }
        }
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
