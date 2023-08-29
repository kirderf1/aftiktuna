use crate::core::position::Pos;
use crate::view::name::NameData;
use crate::view::Messages;
use hecs::{CommandBuffer, ComponentError, Entity, World};
use serde::{Deserialize, Serialize};
use std::cmp::min;

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Health {
    value: f32,
    max: f32,
}

impl Health {
    pub fn with_max(stats: &Stats) -> Health {
        let max = f32::from(4 + stats.endurance * 2);
        Health { max, value: max }
    }

    pub fn is_alive(&self) -> bool {
        self.value > 0.0
    }

    pub fn is_dead(&self) -> bool {
        !self.is_alive()
    }

    pub fn is_hurt(&self) -> bool {
        self.value < self.max
    }

    pub fn as_fraction(&self) -> f32 {
        self.value / self.max
    }

    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.value -= damage;
        self.value <= 0.0
    }

    pub fn restore_fraction(&mut self, fraction: f32) {
        let value = self.value + self.max * fraction;
        if value < self.max {
            self.value = value
        } else {
            self.value = self.max
        }
    }

    #[allow(dead_code)]
    pub fn restore_to_full(&mut self) {
        self.value = self.max
    }
}

pub fn is_alive(entity: Entity, world: &World) -> bool {
    match world.get::<&Health>(entity) {
        Ok(health) => health.is_alive(),
        Err(ComponentError::MissingComponent(_)) => true,
        Err(ComponentError::NoSuchEntity) => false,
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn need_rest(&self) -> bool {
        self.dodge_stamina < self.max
    }

    pub fn need_more_rest(&self) -> bool {
        self.dodge_stamina + 1 < self.max
    }

    pub fn as_fraction(&self) -> f32 {
        f32::from(self.dodge_stamina) / f32::from(self.max)
    }

    pub fn on_dodge_attempt(&mut self) {
        self.dodge_stamina -= 3;
    }
}

#[derive(Serialize, Deserialize)]
pub struct LowHealth;

pub fn detect_low_health(world: &mut World, messages: &mut Messages, character: Entity) {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let mut command_buffer = CommandBuffer::new();
    for (entity, (pos, health)) in world.query::<(&Pos, &Health)>().iter() {
        let has_tag = world.get::<&LowHealth>(entity).is_ok();
        let visible_low_health = pos.is_in(area) && health.as_fraction() < 0.5;
        if has_tag && !visible_low_health {
            command_buffer.remove_one::<LowHealth>(entity);
        }
        if !has_tag && visible_low_health && health.is_alive() {
            command_buffer.insert_one(entity, LowHealth);
            if entity != character {
                messages.add(format!(
                    "{} is badly hurt.",
                    NameData::find(world, entity).definite()
                ));
            }
        }
    }
    command_buffer.run_on(world);
}

#[derive(Serialize, Deserialize)]
pub struct LowStamina;

pub fn detect_low_stamina(world: &mut World, messages: &mut Messages, character: Entity) {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let mut command_buffer = CommandBuffer::new();
    for (entity, (pos, stamina, health)) in world.query::<(&Pos, &Stamina, &Health)>().iter() {
        let has_tag = world.get::<&LowStamina>(entity).is_ok();
        let visible_low_stamina = pos.is_in(area) && stamina.as_fraction() < 0.6;
        if has_tag && !visible_low_stamina {
            command_buffer.remove_one::<LowStamina>(entity);
        }
        if !has_tag && visible_low_stamina && health.is_alive() {
            command_buffer.insert_one(entity, LowStamina);
            messages.add(format!(
                "{} is growing exhausted from dodging attacks.",
                NameData::find(world, entity).definite()
            ));
        }
    }
    command_buffer.run_on(world);
}
