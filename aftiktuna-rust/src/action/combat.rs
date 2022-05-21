use crate::action::{item, Aftik};
use crate::position::{try_move, Pos};
use crate::status::{Health, Stats};
use crate::view::DisplayInfo;
use hecs::{Entity, World};

#[derive(Debug)]
pub struct IsFoe;

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

    let killed = hit(world, target, get_attack_damage(world, attacker));

    if killed {
        if world.get::<Aftik>(target).is_err() {
            world.despawn(target).unwrap();
        }
        Ok(format!(
            "{} attacked and killed {}.",
            attacker_name, target_name
        ))
    } else {
        Ok(format!("{} attacked {}.", attacker_name, target_name))
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
    get_weapon_damage(world) * strength_mod
}

fn get_weapon_damage(world: &World) -> f32 {
    item::get_wielded(world)
        .and_then(|item| world.get::<Weapon>(item).map(|weapon| weapon.0).ok())
        .unwrap_or(2.0)
}
