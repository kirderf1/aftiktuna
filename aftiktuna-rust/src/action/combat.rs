use crate::action::Aftik;
use crate::position::{try_move, Pos};
use crate::view::DisplayInfo;
use hecs::{Entity, World};

#[derive(Debug)]
pub struct IsFoe;

#[derive(Debug)]
pub struct Health {
    value: f32,
    max: f32,
}

impl Health {
    pub fn with_max(stats: &Stats) -> Health {
        let max = f32::from(4 + stats.endurance * 2);
        Health { max, value: max }
    }

    pub fn as_fraction(&self) -> f32 {
        self.value / self.max
    }

    pub fn is_alive(entity: Entity, world: &World) -> bool {
        if let Ok(health) = world.get::<Health>(entity) {
            health.value > 0.0
        } else {
            true
        }
    }
}

pub struct Stats {
    pub strength: i16,
    pub endurance: i16,
    pub agility: i16,
}

impl Stats {
    pub fn new(strength: i16, endurance: i16, agility: i16) -> Stats {
        Stats {
            strength,
            endurance,
            agility,
        }
    }
}

pub fn attack(world: &mut World, attacker: Entity, target: Entity) -> Result<String, String> {
    let attacker_name = DisplayInfo::find_definite_name(world, attacker);
    let target_name = DisplayInfo::find_definite_name(world, target);
    let attacker_pos = *world.get::<Pos>(attacker).unwrap();
    let target_pos = *world.get::<Pos>(target).unwrap();

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
        health.value -= damage;
        health.value <= 0.0
    } else {
        false
    }
}

fn get_attack_damage(world: &World, aftik: Entity) -> f32 {
    let strength = world.get::<Stats>(aftik).unwrap().strength;
    let strength_mod = f32::from(strength + 2) / 6.0;
    2.0 * strength_mod
}
