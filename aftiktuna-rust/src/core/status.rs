use super::name::NameWithAttribute;
use super::position::Pos;
use crate::view::text::Messages;
use hecs::{CommandBuffer, Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub strength: i16,
    pub endurance: i16,
    pub agility: i16,
    pub luck: i16,
}

impl Stats {
    pub fn new(strength: i16, endurance: i16, agility: i16, luck: i16) -> Stats {
        Stats {
            strength,
            endurance,
            agility,
            luck,
        }
    }

    pub fn try_change_in_bounds(
        &mut self,
        changes: StatChanges,
    ) -> Result<ChangedStats, OutsideBounds> {
        if changes.strength != 0 && !(1..=10).contains(&(self.strength + changes.strength))
            || changes.endurance != 0 && !(1..=10).contains(&(self.endurance + changes.endurance))
            || changes.agility != 0 && !(1..=10).contains(&(self.agility + changes.agility))
            || changes.luck != 0 && !(0..=10).contains(&(self.luck + changes.luck))
        {
            return Err(OutsideBounds);
        }
        self.strength += changes.strength;
        self.endurance += changes.endurance;
        self.agility += changes.agility;
        self.luck += changes.luck;
        Ok(ChangedStats)
    }

    pub fn agility_for_dodging(&self, entity_ref: EntityRef) -> i16 {
        if Trait::GoodDodger.ref_has_trait(entity_ref) {
            self.agility + 5
        } else {
            self.agility
        }
    }
}

pub struct ChangedStats;

pub struct OutsideBounds;

#[derive(Debug, Clone, Copy, Default)]
pub struct StatChanges {
    pub strength: i16,
    pub endurance: i16,
    pub agility: i16,
    pub luck: i16,
}

impl StatChanges {
    pub const DEFAULT: Self = Self {
        strength: 0,
        endurance: 0,
        agility: 0,
        luck: 0,
    };

    pub fn try_apply(self, entity_ref: EntityRef) -> Option<ChangedStats> {
        entity_ref
            .get::<&mut Stats>()
            .and_then(|mut stats| stats.try_change_in_bounds(self).ok())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Traits(HashSet<Trait>);

impl Traits {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn has_traits(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn sorted_iter(&self) -> impl Iterator<Item = Trait> + use<> {
        let mut traits = self.0.iter().copied().collect::<Vec<_>>();
        traits.sort();
        traits.into_iter()
    }
}

impl<T: IntoIterator<Item = Trait>> From<T> for Traits {
    fn from(value: T) -> Self {
        Self(value.into_iter().collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trait {
    GoodDodger,
    FastHealer,

    Fragile,
    BigEater,
}

impl Trait {
    pub fn name(self) -> &'static str {
        match self {
            Trait::BigEater => "[Big Eater]",
            Trait::FastHealer => "[Fast Healer]",
            Trait::Fragile => "[Fragile]",
            Trait::GoodDodger => "[Good Dodger]",
        }
    }

    pub fn has_trait(self, entity: Entity, world: &World) -> bool {
        world
            .entity(entity)
            .map_or(false, |entity_ref| self.ref_has_trait(entity_ref))
    }

    pub fn ref_has_trait(self, entity: EntityRef) -> bool {
        entity
            .get::<&Traits>()
            .map_or(false, |traits| traits.0.contains(&self))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Health {
    value: f32,
}

impl Health {
    pub fn from_fraction(fraction: f32) -> Self {
        Self {
            value: fraction.clamp(0., 1.),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.value > 0.
    }

    pub fn is_dead(&self) -> bool {
        !self.is_alive()
    }

    pub fn is_hurt(&self) -> bool {
        self.value < 1.
    }

    pub fn is_badly_hurt(&self) -> bool {
        self.value < 0.5
    }

    pub fn as_fraction(&self) -> f32 {
        self.value
    }

    pub fn take_damage(&mut self, mut damage: f32, entity_ref: EntityRef) -> bool {
        if Trait::Fragile.ref_has_trait(entity_ref) {
            damage *= 1.33;
        }
        let endurance = entity_ref
            .get::<&Stats>()
            .map_or(1, |stats| stats.endurance);
        self.value -= damage / f32::from(4 + endurance * 2);
        self.value <= 0.0
    }

    pub fn restore_fraction(&mut self, fraction: f32) {
        self.value = f32::min(1., self.value + fraction)
    }

    #[allow(dead_code)]
    pub fn restore_to_full(&mut self) {
        self.value = 1.
    }
}

pub fn is_alive(entity: Entity, world: &World) -> bool {
    match world.entity(entity) {
        Ok(entity_ref) => is_alive_ref(entity_ref),
        Err(hecs::NoSuchEntity) => false,
    }
}

pub fn is_alive_ref(entity_ref: EntityRef) -> bool {
    match entity_ref.get::<&Health>() {
        Some(health) => health.is_alive(),
        None => true,
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
        let entity_ref = world.entity(entity).unwrap();
        let has_tag = entity_ref.has::<LowHealth>();
        let visible_low_health = pos.is_in(area) && health.is_badly_hurt();
        if has_tag && !visible_low_health {
            command_buffer.remove_one::<LowHealth>(entity);
        }
        if !has_tag && visible_low_health && health.is_alive() {
            command_buffer.insert_one(entity, LowHealth);
            if entity != character {
                messages.add(format!(
                    "{the_entity} is badly hurt.",
                    the_entity = NameWithAttribute::lookup_by_ref(entity_ref).definite()
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
        let entity_ref = world.entity(entity).unwrap();
        let has_tag = entity_ref.has::<LowStamina>();
        let visible_low_stamina = pos.is_in(area) && stamina.as_fraction() < 0.6;
        if has_tag && !visible_low_stamina {
            command_buffer.remove_one::<LowStamina>(entity);
        }
        if !has_tag && visible_low_stamina && health.is_alive() {
            command_buffer.insert_one(entity, LowStamina);
            messages.add(format!(
                "{the_entity} is growing exhausted from dodging attacks.",
                the_entity = NameWithAttribute::lookup_by_ref(entity_ref).definite()
            ));
        }
    }
    command_buffer.run_on(world);
}

pub fn get_food_heal_fraction(entity_ref: EntityRef) -> f32 {
    if Trait::FastHealer.ref_has_trait(entity_ref) {
        0.5
    } else {
        0.33
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IsStunned;
