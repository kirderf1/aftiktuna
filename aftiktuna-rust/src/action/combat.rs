use crate::action::{item, Aftik};
use crate::position::{try_move, Pos};
use crate::status::{Health, Stamina, Stats};
use crate::view::DisplayInfo;
use fastrand::Rng;
use hecs::{Component, Entity, World};

#[derive(Debug)]
pub struct IsFoe;

#[derive(Clone)]
pub enum Target {
    Aftik,
    Foe,
}

pub fn attack_nearest(
    world: &mut World,
    attacker: Entity,
    target: Target,
) -> Result<String, String> {
    let pos = *world.get::<Pos>(attacker).unwrap();
    let target = match target {
        Target::Aftik => find_closest::<Aftik>(world, pos),
        Target::Foe => find_closest::<IsFoe>(world, pos),
    };

    match target {
        Some(target) => attack(world, attacker, target),
        None => Err("There is no appropriate target to attack here.".to_string()),
    }
}

fn find_closest<T: Component>(world: &mut World, pos: Pos) -> Option<Entity> {
    world
        .query::<&Pos>()
        .with::<T>()
        .iter()
        .filter(|(_, other_pos)| other_pos.is_in(pos.get_area()))
        .min_by_key(|(_, other_pos)| other_pos.distance_to(pos))
        .map(|data| data.0)
}

pub fn attack(world: &mut World, attacker: Entity, target: Entity) -> Result<String, String> {
    let attacker_name = DisplayInfo::find_definite_name(world, attacker);
    let target_name = DisplayInfo::find_definite_name(world, target);
    let attacker_pos = *world
        .get::<Pos>(attacker)
        .expect("Expected attacker to have a position");
    let target_pos = *world.get::<Pos>(target).map_err(|_| {
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

    try_move(
        world,
        attacker,
        target_pos.get_adjacent_towards(attacker_pos),
    )?;

    let hit_type = roll_hit(world, attacker, target, &mut Rng::new());

    if hit_type == HitType::Dodge {
        return Ok(format!(
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
        if world.get::<Aftik>(target).is_err() {
            world.despawn(target).unwrap();
        }

        if hit_type == HitType::GrazingHit {
            Ok(format!(
                "{}'s attack grazed and killed {}.",
                attacker_name, target_name
            ))
        } else {
            Ok(format!(
                "{} got a direct hit on and killed {}.",
                attacker_name, target_name
            ))
        }
    } else if hit_type == HitType::GrazingHit {
        Ok(format!(
            "{}'s attack grazed {}.",
            attacker_name, target_name
        ))
    } else {
        Ok(format!(
            "{} got a direct hit on {}.",
            attacker_name, target_name
        ))
    }
}

pub fn hit(world: &mut World, target: Entity, damage: f32) -> bool {
    if let Ok(mut health) = world.get_mut::<Health>(target) {
        health.take_damage(damage)
    } else {
        false
    }
}

#[derive(Debug)]
pub struct Weapon(pub f32);

fn get_attack_damage(world: &World, attacker: Entity) -> f32 {
    let strength = world
        .get::<Stats>(attacker)
        .expect("Expected attacker to have stats attached")
        .strength;
    let strength_mod = f32::from(strength + 2) / 6.0;
    get_weapon_damage(world, attacker) * strength_mod
}

fn get_weapon_damage(world: &World, attacker: Entity) -> f32 {
    item::get_wielded(world, attacker)
        .and_then(|item| world.get::<Weapon>(item).map(|weapon| weapon.0).ok())
        .unwrap_or(2.0)
}

fn roll_hit(world: &mut World, attacker: Entity, defender: Entity, rng: &mut Rng) -> HitType {
    let mut stamina = world.get_mut::<Stamina>(defender).unwrap();
    let stamina_factor = stamina.as_fraction();
    if stamina_factor > 0.0 {
        stamina.on_dodge_attempt();
        let dodge_factor = stamina_factor * get_dodge_factor(world, attacker, defender);
        // Yes, this looks slightly odd. This is meant to act as a d20 integer roll,
        // which is converted to a float only to be compared against the float factor.
        let hit_roll = f32::from(rng.i16(1..=20));

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
    let hit_agility = world.get::<Stats>(attacker).unwrap().agility;
    let dodge_agility = world.get::<Stats>(defender).unwrap().agility;
    f32::from(2 * dodge_agility - hit_agility)
}

#[derive(Debug, Eq, PartialEq)]
enum HitType {
    DirectHit,
    GrazingHit,
    Dodge,
}
