use crate::action::combat::IsFoe;
use crate::action::door::GoingToShip;
use crate::action::{combat, Action, CrewMember};
use crate::core::item::Weapon;
use crate::core::position::Pos;
use crate::core::{inventory, status};
use crate::view::name::NameData;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Intention {
    Wield(Entity),
    Force(Entity),
}

pub fn prepare_intentions(world: &mut World) {
    let crew_members = world
        .query::<()>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for crew_member in crew_members {
        prepare_intention(world, crew_member);

        if world.satisfies::<&GoingToShip>(crew_member).unwrap() {
            world.insert_one(crew_member, Action::GoToShip).unwrap();
        }
    }
}

fn prepare_intention(world: &mut World, crew_member: Entity) {
    fn pick_intention(world: &mut World, crew_member: Entity) -> Option<Intention> {
        let weapon_damage = combat::get_weapon_damage(world, crew_member);

        for item in inventory::get_inventory(world, crew_member) {
            if let Ok(weapon) = world.get::<&Weapon>(item) {
                if weapon.0 > weapon_damage {
                    return Some(Intention::Wield(item));
                }
            }
        }
        None
    }

    if let Some(intention) = pick_intention(world, crew_member) {
        world.insert_one(crew_member, intention).unwrap();
    }
}

pub fn tick(world: &mut World) {
    let foes = world
        .query::<()>()
        .with::<&IsFoe>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for foe in foes {
        foe_ai(world, foe);
    }

    let aftiks = world
        .query::<()>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for aftik in aftiks {
        aftik_ai(world, aftik);
    }
}

fn foe_ai(world: &mut World, foe: Entity) {
    if status::is_alive(foe, world) && world.get::<&Action>(foe).is_err() {
        if let Some(action) = pick_foe_action(world, foe) {
            world.insert_one(foe, action).unwrap();
        }
    }
}

fn pick_foe_action(world: &World, foe: Entity) -> Option<Action> {
    let pos = *world.get::<&Pos>(foe).ok()?;
    let targets = world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(_, aftik_pos)| aftik_pos.is_in(pos.get_area()))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if !targets.is_empty() {
        Some(Action::Attack(targets))
    } else {
        None
    }
}

fn aftik_ai(world: &mut World, crew_member: Entity) {
    let intention = world.remove_one::<Intention>(crew_member).ok();
    if status::is_alive(crew_member, world) && world.get::<&Action>(crew_member).is_err() {
        if let Some(action) = pick_aftik_action(world, crew_member, intention) {
            world.insert_one(crew_member, action).unwrap();
        }
    }
}

fn pick_aftik_action(world: &World, aftik: Entity, intention: Option<Intention>) -> Option<Action> {
    let pos = *world.get::<&Pos>(aftik).ok()?;
    let foes = world
        .query::<&Pos>()
        .with::<&IsFoe>()
        .iter()
        .filter(|(_, foe_pos)| foe_pos.is_in(pos.get_area()))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if !foes.is_empty() {
        return Some(Action::Attack(foes));
    }

    if let Some(intention) = intention {
        return Some(match intention {
            Intention::Wield(item) => Action::Wield(item, NameData::find(world, item)),
            Intention::Force(door) => Action::ForceDoor(door, true),
        });
    }

    None
}

pub fn is_requesting_wait(world: &World, entity: Entity) -> bool {
    world.get::<&Intention>(entity).is_ok()
}
