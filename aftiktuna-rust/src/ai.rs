use crate::action::Action;
use crate::core::item::Weapon;
use crate::core::name::NameData;
use crate::core::position::Pos;
use crate::core::{self, inventory, status, CrewMember, GoingToShip, Hostile};
use hecs::{CommandBuffer, Entity, EntityRef, Or, World};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Intention {
    Wield(Entity),
    Force(Entity),
}

pub fn prepare_intentions(world: &mut World) {
    let mut buffer = CommandBuffer::new();

    for (crew_member, _) in world.query::<()>().with::<&CrewMember>().iter() {
        if let Some(intention) = pick_intention(crew_member, world) {
            buffer.insert_one(crew_member, intention);
        };

        if world.satisfies::<&GoingToShip>(crew_member).unwrap() {
            buffer.insert_one(crew_member, Action::GoToShip);
        }
    }

    buffer.run_on(world);
}

fn pick_intention(crew_member: Entity, world: &World) -> Option<Intention> {
    let weapon_damage = core::get_wielded_weapon_modifier(world, crew_member);

    for item in inventory::get_inventory(world, crew_member) {
        if let Ok(weapon) = world.get::<&Weapon>(item) {
            if weapon.0 > weapon_damage {
                return Some(Intention::Wield(item));
            }
        }
    }

    None
}

pub fn tick(world: &mut World) {
    let mut buffer = CommandBuffer::new();

    for (entity, _) in world
        .query::<()>()
        .with::<Or<&CrewMember, &Hostile>>()
        .iter()
    {
        let entity_ref = world.entity(entity).unwrap();
        if status::is_alive_ref(entity_ref) && !entity_ref.satisfies::<&Action>() {
            let action = pick_action(entity_ref, world).unwrap_or(Action::Wait);

            buffer.insert_one(entity, action);
        };
    }

    world
        .query::<()>()
        .with::<&Intention>()
        .iter()
        .for_each(|(entity, _)| buffer.remove_one::<Intention>(entity));

    buffer.run_on(world);
}

fn pick_action(entity_ref: EntityRef, world: &World) -> Option<Action> {
    if let Some(hostile) = entity_ref.get::<&Hostile>() {
        pick_foe_action(entity_ref, &hostile, world)
    } else if entity_ref.satisfies::<&CrewMember>() {
        pick_crew_action(entity_ref, world)
    } else {
        None
    }
}

fn pick_foe_action(entity_ref: EntityRef, hostile: &Hostile, world: &World) -> Option<Action> {
    if hostile.aggressive {
        let area = entity_ref.get::<&Pos>()?.get_area();

        let targets = world
            .query::<&Pos>()
            .with::<&CrewMember>()
            .iter()
            .filter(|&(crew, crew_pos)| crew_pos.is_in(area) && status::is_alive(crew, world))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        if !targets.is_empty() {
            return Some(Action::Attack(targets));
        }
    }

    None
}

fn pick_crew_action(entity_ref: EntityRef, world: &World) -> Option<Action> {
    let area = entity_ref.get::<&Pos>()?.get_area();

    let foes = world
        .query::<(&Pos, &Hostile)>()
        .iter()
        .filter(|&(foe, (foe_pos, hostile))| {
            foe_pos.is_in(area) && status::is_alive(foe, world) && hostile.aggressive
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if !foes.is_empty() {
        return Some(Action::Attack(foes));
    }

    if let Some(intention) = entity_ref.get::<&Intention>() {
        return Some(match *intention {
            Intention::Wield(item) => Action::Wield(item, NameData::find(world, item)),
            Intention::Force(door) => Action::ForceDoor(door, true),
        });
    }

    None
}

pub fn is_requesting_wait(world: &World, entity: Entity) -> bool {
    world.get::<&Intention>(entity).is_ok()
}
