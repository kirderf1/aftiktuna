use crate::action;
use crate::action::{item, CrewMember};
use crate::core::item::Weapon;
use crate::core::position::{try_move_adjacent, Pos};
use crate::core::GameState;
use crate::status::{Health, Stamina, Stats};
use crate::view::NameData;
use hecs::{Component, Entity, World};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Serialize, Deserialize)]
pub struct IsFoe;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Target {
    Aftik,
    Foe,
}

pub fn attack_nearest(state: &mut GameState, attacker: Entity, target: Target) -> action::Result {
    let world = &mut state.world;
    let pos = *world.get::<&Pos>(attacker).unwrap();
    let target = match target {
        Target::Aftik => find_closest::<CrewMember, _>(world, pos, &mut state.rng),
        Target::Foe => find_closest::<IsFoe, _>(world, pos, &mut state.rng),
    };

    match target {
        Some(target) => attack(state, attacker, target),
        None => action::silent_ok(),
    }
}

fn find_closest<T: Component, R: Rng>(world: &mut World, pos: Pos, rng: &mut R) -> Option<Entity> {
    let targets = world
        .query::<&Pos>()
        .with::<&T>()
        .iter()
        .filter(|(_, other_pos)| other_pos.is_in(pos.get_area()))
        // collects the closest targets and also maps them to just the entity in one
        .fold(
            (usize::MAX, Vec::new()),
            |mut partial, (entity, other_pos)| {
                let distance = other_pos.distance_to(pos);
                match distance.cmp(&partial.0) {
                    Ordering::Less => (distance, vec![entity]),
                    Ordering::Equal => {
                        partial.1.push(entity);
                        partial
                    }
                    Ordering::Greater => partial,
                }
            },
        )
        .1;
    if targets.is_empty() {
        None
    } else {
        Some(targets[rng.gen_range(0..targets.len())])
    }
}

pub fn attack(state: &mut GameState, attacker: Entity, target: Entity) -> action::Result {
    let world = &mut state.world;
    let attacker_name = NameData::find(world, attacker).definite();
    let target_name = NameData::find(world, target).definite();
    let attacker_pos = *world
        .get::<&Pos>(attacker)
        .expect("Expected attacker to have a position");
    let target_pos = *world.get::<&Pos>(target).map_err(|_| {
        format!(
            "{} disappeared before {} could attack.",
            target_name, attacker_name
        )
    })?;

    if attacker_pos.get_area() != target_pos.get_area() {
        return Err(format!(
            "{} left before {} could attack.",
            target_name, attacker_name
        ));
    }

    try_move_adjacent(world, attacker, target_pos)?;

    let hit_type = roll_hit(world, attacker, target, &mut state.rng);

    if hit_type == HitType::Dodge {
        return action::ok(format!(
            "{} dodged {}'s attack.",
            target_name, attacker_name
        ));
    }

    let damage_factor = if hit_type == HitType::GrazingHit {
        0.5
    } else {
        1.0
    };

    let killed = hit(
        world,
        target,
        damage_factor * get_attack_damage(world, attacker),
    );

    if killed {
        if world.get::<&CrewMember>(target).is_err() {
            world.despawn(target).unwrap();
        }

        if hit_type == HitType::GrazingHit {
            action::ok(format!(
                "{}'s attack grazed and killed {}.",
                attacker_name, target_name
            ))
        } else {
            action::ok(format!(
                "{} got a direct hit on and killed {}.",
                attacker_name, target_name
            ))
        }
    } else if hit_type == HitType::GrazingHit {
        action::ok(format!(
            "{}'s attack grazed {}.",
            attacker_name, target_name
        ))
    } else {
        action::ok(format!(
            "{} got a direct hit on {}.",
            attacker_name, target_name
        ))
    }
}

pub fn hit(world: &mut World, target: Entity, damage: f32) -> bool {
    if let Ok(mut health) = world.get::<&mut Health>(target) {
        health.take_damage(damage)
    } else {
        false
    }
}

fn get_attack_damage(world: &World, attacker: Entity) -> f32 {
    let strength = world
        .get::<&Stats>(attacker)
        .expect("Expected attacker to have stats attached")
        .strength;
    let strength_mod = f32::from(strength + 2) / 6.0;
    get_weapon_damage(world, attacker) * strength_mod
}

pub fn get_weapon_damage(world: &World, attacker: Entity) -> f32 {
    item::get_wielded(world, attacker)
        .and_then(|item| world.get::<&Weapon>(item).map(|weapon| weapon.0).ok())
        .unwrap_or(2.0)
}

fn roll_hit(world: &mut World, attacker: Entity, defender: Entity, rng: &mut impl Rng) -> HitType {
    let mut stamina = world.get::<&mut Stamina>(defender).unwrap();
    let stamina_factor = stamina.as_fraction();
    if stamina_factor > 0.0 {
        stamina.on_dodge_attempt();
        let dodge_factor = stamina_factor * get_dodge_factor(world, attacker, defender);
        // Yes, this looks slightly odd. This is meant to act as a d20 integer roll,
        // which is converted to a float only to be compared against the float factor.
        let hit_roll = f32::from(rng.gen_range::<i16, _>(1..=20));

        if dodge_factor > hit_roll + 5.0 {
            HitType::Dodge
        } else if dodge_factor > hit_roll {
            HitType::GrazingHit
        } else {
            HitType::DirectHit
        }
    } else {
        HitType::DirectHit
    }
}

fn get_dodge_factor(world: &World, attacker: Entity, defender: Entity) -> f32 {
    let hit_agility = world.get::<&Stats>(attacker).unwrap().agility;
    let dodge_agility = world.get::<&Stats>(defender).unwrap().agility;
    f32::from(2 * dodge_agility - hit_agility)
}

#[derive(Debug, Eq, PartialEq)]
enum HitType {
    DirectHit,
    GrazingHit,
    Dodge,
}
