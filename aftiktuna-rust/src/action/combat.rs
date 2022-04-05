use crate::position::{try_move_aftik, Pos};
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
        let max = (4 + stats.endurance * 2) as f32;
        Health { max, value: max }
    }

    pub fn as_fraction(&self) -> f32 {
        self.value / self.max
    }
}

pub struct Stats {
    pub strength: i32,
    pub endurance: i32,
}

impl Stats {
    pub fn new(strength: i32, endurance: i32) -> Stats {
        Stats {
            strength,
            endurance,
        }
    }
}

pub fn attack(world: &mut World, aftik: Entity, target: Entity) -> Result<String, String> {
    let name = world.get::<DisplayInfo>(target).unwrap().name().to_string();
    let target_pos = *world.get::<Pos>(target).unwrap();
    let aftik_pos = *world.get::<Pos>(aftik).unwrap();

    try_move_aftik(world, aftik, target_pos.get_adjacent_towards(aftik_pos))?;

    let killed = hit(world, target, get_attack_damage(world, aftik));

    if killed {
        world.despawn(target).unwrap();
        Ok(format!("You attacked and killed the {}.", name))
    } else {
        Ok(format!("You attacked the {}.", name))
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
    let strength_mod = (strength + 2) as f32 / 6.0;
    2.0 * strength_mod
}
