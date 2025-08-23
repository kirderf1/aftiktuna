use crate::action::Action;
use crate::action::item::UseAction;
use crate::asset::NounDataMap;
use crate::core::item::ItemType;
use crate::core::name::NameData;
use crate::core::position::{self, Pos};
use crate::core::{
    self, AttackKind, BadlyHurtBehavior, Character, CrewMember, Door, Hostile, ObservationTarget,
    RepeatingAction, Species, Waiting, Wandering, area, inventory, status,
};
use hecs::{CommandBuffer, Entity, EntityRef, Or, World};
use rand::Rng;
use rand::seq::{IndexedRandom, IteratorRandom};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize)]
pub enum Intention {
    Wield(Entity),
    Force(Entity),
    UseMedkit(Entity),
}

pub fn prepare_intentions(world: &mut World) {
    let mut buffer = CommandBuffer::new();

    for (crew_member, _) in world
        .query::<()>()
        .with::<(&CrewMember, &Character)>()
        .iter()
    {
        if let Some(intention) = pick_intention(crew_member, world) {
            buffer.insert_one(crew_member, intention);
        };
    }

    for (crew_member, action) in world
        .query::<&RepeatingAction>()
        .with::<&CrewMember>()
        .iter()
    {
        if action.cancel_if_unsafe()
            && let Ok(pos) = world.get::<&Pos>(crew_member)
            && !core::is_safe(world, pos.get_area())
        {
            buffer.remove_one::<RepeatingAction>(crew_member);
        };
    }

    buffer.run_on(world);
}

fn pick_intention(crew_member: Entity, world: &World) -> Option<Intention> {
    if world
        .get::<&status::Health>(crew_member)
        .is_ok_and(|health| health.is_badly_hurt())
    {
        for item in inventory::get_inventory(world, crew_member) {
            if world
                .get::<&ItemType>(item)
                .is_ok_and(|item_type| *item_type == ItemType::Medkit)
            {
                return Some(Intention::UseMedkit(item));
            }
        }
    }

    let current_properties = core::get_active_weapon_properties(world, crew_member);

    for item in inventory::get_inventory(world, crew_member) {
        if let Some(properties) = world
            .get::<&ItemType>(item)
            .ok()
            .and_then(|item_type| item_type.weapon_properties())
            && properties.damage_mod > current_properties.damage_mod
        {
            return Some(Intention::Wield(item));
        }
    }

    None
}

pub fn tick(
    action_map: &mut HashMap<Entity, Action>,
    world: &mut World,
    rng: &mut impl Rng,
    noun_map: &NounDataMap,
) {
    let mut buffer = CommandBuffer::new();

    for (entity, _) in world
        .query::<()>()
        .with::<Or<&CrewMember, &Hostile>>()
        .iter()
    {
        let entity_ref = world.entity(entity).unwrap();
        if status::is_alive_ref(entity_ref) && !action_map.contains_key(&entity) {
            let action = if let Some(action) = entity_ref.get::<&RepeatingAction>() {
                buffer.remove_one::<RepeatingAction>(entity);
                Action::from(*action)
            } else {
                pick_action(entity_ref, world, rng, noun_map).unwrap_or(Action::Wait)
            };

            action_map.insert(entity, action);
        };
    }

    world
        .query::<()>()
        .with::<&Intention>()
        .iter()
        .for_each(|(entity, _)| buffer.remove_one::<Intention>(entity));

    buffer.run_on(world);
}

fn pick_action(
    entity_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
    noun_map: &NounDataMap,
) -> Option<Action> {
    if let Some(hostile) = entity_ref.get::<&Hostile>() {
        pick_foe_action(entity_ref, &hostile, world, rng)
    } else if entity_ref.satisfies::<&CrewMember>() {
        pick_crew_action(entity_ref, world, rng, noun_map)
    } else {
        None
    }
}

fn pick_foe_action(
    entity_ref: EntityRef,
    hostile: &Hostile,
    world: &World,
    rng: &mut impl Rng,
) -> Option<Action> {
    if has_behavior(entity_ref, BadlyHurtBehavior::Fearful)
        && let Some(path) = find_random_unblocked_path(entity_ref, world, rng)
    {
        return Some(Action::EnterDoor(path));
    }

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
            return Some(Action::Attack(
                targets,
                pick_attack_kind(entity_ref, world, rng),
            ));
        }
    }

    if entity_ref.has::<Wandering>() {
        let area = entity_ref.get::<&Pos>()?.get_area();

        let observation_targets = world
            .query::<&Pos>()
            .with::<&ObservationTarget>()
            .iter()
            .filter(|&(_, pos)| pos.is_in(area))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        if let Some(&observation_target) = observation_targets.choose(rng)
            && rng.random_bool(8. / 10.)
        {
            return Some(Action::Examine(observation_target));
        } else if rng.random_bool(1. / 2.)
            && let Some(door) = find_random_unblocked_path(entity_ref, world, rng)
        {
            return Some(Action::EnterDoor(door));
        }
    }

    None
}

fn pick_crew_action(
    entity_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
    noun_map: &NounDataMap,
) -> Option<Action> {
    let entity_pos = *entity_ref.get::<&Pos>()?;

    if has_behavior(entity_ref, BadlyHurtBehavior::Fearful) {
        let is_area_safe =
            area::is_in_ship(entity_pos, world) && core::is_safe(world, entity_pos.get_area());
        if !is_area_safe {
            if let Some(path) = find_path_towards(world, entity_pos.get_area(), |area| {
                area::is_ship(area, world)
            }) && position::check_is_blocked(
                world,
                entity_ref,
                entity_pos,
                *world.get::<&Pos>(path).unwrap(),
            )
            .is_ok()
            {
                return Some(Action::EnterDoor(path));
            } else if let Some(path) = find_random_unblocked_path(entity_ref, world, rng) {
                return Some(Action::EnterDoor(path));
            }
        }
    }

    let intention = entity_ref.get::<&Intention>();
    if let Some(&Intention::UseMedkit(item)) = intention.as_deref() {
        return Some(UseAction { item }.into());
    }

    if entity_ref
        .get::<&Waiting>()
        .is_some_and(|waiting| waiting.at_ship)
        && !area::is_in_ship(entity_pos, world)
        && let Some(path) = find_path_towards(world, entity_pos.get_area(), |area| {
            area::is_ship(area, world)
        })
        && position::check_is_blocked(
            world,
            entity_ref,
            entity_pos,
            *world.get::<&Pos>(path).unwrap(),
        )
        .is_ok()
    {
        return Some(Action::EnterDoor(path));
    }

    let foes = world
        .query::<(&Pos, &Hostile)>()
        .iter()
        .filter(|&(foe, (foe_pos, hostile))| {
            foe_pos.is_in(entity_pos.get_area())
                && status::is_alive(foe, world)
                && hostile.aggressive
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if !foes.is_empty() {
        return Some(Action::Attack(
            foes,
            pick_attack_kind(entity_ref, world, rng),
        ));
    }

    if let Some(intention) = intention {
        match *intention {
            Intention::Wield(item) => {
                return Some(Action::Wield(item, NameData::find(world, item, noun_map)));
            }
            Intention::Force(door) => return Some(Action::ForceDoor(door, true)),
            _ => {}
        };
    }

    None
}

pub fn pick_attack_kind(attacker_ref: EntityRef, world: &World, rng: &mut impl Rng) -> AttackKind {
    let available_kinds = core::get_active_weapon_properties(world, attacker_ref.entity())
        .attack_set
        .available_kinds();

    if has_behavior(attacker_ref, BadlyHurtBehavior::Determined)
        && available_kinds.contains(&AttackKind::Rash)
    {
        AttackKind::Rash
    } else {
        available_kinds
            .choose(rng)
            .copied()
            .unwrap_or(AttackKind::Light)
    }
}

pub fn is_requesting_wait(world: &World, entity: Entity) -> bool {
    world
        .satisfies::<hecs::Or<&Intention, &status::IsStunned>>(entity)
        .unwrap_or(false)
}

fn find_random_unblocked_path(
    entity_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
) -> Option<Entity> {
    let entity_pos = *entity_ref.get::<&Pos>()?;
    world
        .query::<&Pos>()
        .with::<&Door>()
        .iter()
        .filter(|&(_, path_pos)| {
            path_pos.is_in(entity_pos.get_area())
                && position::check_is_blocked(world, entity_ref, entity_pos, *path_pos).is_ok()
        })
        .choose(rng)
        .map(|(path, _)| path)
}

fn has_behavior(entity_ref: EntityRef, behavior: BadlyHurtBehavior) -> bool {
    entity_ref
        .get::<&status::Health>()
        .is_some_and(|health| health.is_badly_hurt())
        && entity_ref
            .get::<&Species>()
            .and_then(|species| species.badly_hurt_behavior())
            == Some(behavior)
}

struct PathSearchEntry {
    path: Entity,
    area: Entity,
}

impl PathSearchEntry {
    fn start(path_entity: Entity, path: &Door) -> Self {
        Self {
            path: path_entity,
            area: path.destination.get_area(),
        }
    }

    fn next(&self, path: &Door) -> Self {
        Self {
            path: self.path,
            area: path.destination.get_area(),
        }
    }
}

pub fn find_path_towards(
    world: &World,
    area: Entity,
    predicate: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    let mut entries = world
        .query::<(&Pos, &Door)>()
        .iter()
        .filter(|&(_, (pos, _))| pos.is_in(area))
        .map(|(entity, (_, path))| PathSearchEntry::start(entity, path))
        .collect::<Vec<_>>();
    let mut checked_areas = HashSet::from([area]);

    while !entries.is_empty() {
        let mut new_entries = vec![];
        for entry in entries {
            if checked_areas.insert(entry.area) {
                if predicate(entry.area) {
                    return Some(entry.path);
                }
                new_entries.extend(
                    world
                        .query::<(&Pos, &Door)>()
                        .iter()
                        .filter(|&(_, (pos, _))| pos.is_in(entry.area))
                        .map(|(_, (_, path))| entry.next(path)),
                );
            }
        }
        entries = new_entries;
    }

    None
}
