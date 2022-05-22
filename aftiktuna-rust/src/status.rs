use hecs::{ComponentError, Entity, World};
use std::cmp::min;

#[derive(Debug)]
pub struct Health {
    value: f32,
    max: f32,
}

#[derive(Debug)]
pub struct Stamina {
    dodge_stamina: i16,
    max: i16,
}

impl Stamina {
    pub fn with_max(stats: &Stats) -> Stamina {
        let max = 4 + stats.endurance * 2;
        Stamina {
            max,
            dodge_stamina: max,
        }
    }

    pub fn tick(&mut self) {
        self.dodge_stamina = min(self.dodge_stamina + 1, self.max);
    }

    pub fn as_fraction(&self) -> f32 {
        f32::from(self.dodge_stamina) / f32::from(self.max)
    }

    pub fn on_dodge_attempt(&mut self) {
        self.dodge_stamina -= 3;
    }
}

impl Health {
    pub fn with_max(stats: &Stats) -> Health {
        let max = f32::from(4 + stats.endurance * 2);
        Health { max, value: max }
    }

    pub fn as_fraction(&self) -> f32 {
        self.value / self.max
    }

    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.value -= damage;
        self.value <= 0.0
    }
}

pub fn is_alive(entity: Entity, world: &World) -> bool {
    match world.get::<Health>(entity) {
        Ok(health) => health.value > 0.0,
        Err(ComponentError::MissingComponent(_)) => true,
        Err(ComponentError::NoSuchEntity) => false,
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
