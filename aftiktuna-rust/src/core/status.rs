use super::Species;
use super::behavior::BadlyHurtBehavior;
use super::name::NameWithAttribute;
use super::position::Pos;
use crate::core::CrewMember;
use crate::core::behavior::{Character, CrewLossMemory};
use crate::core::item::ItemType;
use crate::view;
use hecs::{CommandBuffer, Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::Display;

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
        let morale = entity_ref
            .get::<&Morale>()
            .as_deref()
            .copied()
            .unwrap_or_default();
        self.agility
            + if Trait::GoodDodger.ref_has_trait(entity_ref) {
                5
            } else {
                0
            }
            + morale.dodge_mod()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureAttribute {
    Muscular,
    Bulky,
    Agile,
}

impl CreatureAttribute {
    pub fn variants() -> &'static [Self] {
        use CreatureAttribute::*;
        &[Muscular, Bulky, Agile]
    }

    pub fn adjust_stats(self, stats: &mut Stats) {
        match self {
            CreatureAttribute::Muscular => {
                stats.strength += 3;
                stats.luck -= 1;
            }
            CreatureAttribute::Bulky => {
                stats.endurance += 3;
                stats.agility -= 1;
            }
            CreatureAttribute::Agile => {
                stats.agility += 3;
                stats.endurance -= 1;
            }
        }
    }

    pub fn as_adjective(self) -> &'static str {
        match self {
            CreatureAttribute::Muscular => "muscular",
            CreatureAttribute::Bulky => "bulky",
            CreatureAttribute::Agile => "agile",
        }
    }
}

impl Display for CreatureAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_adjective(), f)
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
            .is_ok_and(|entity_ref| self.ref_has_trait(entity_ref))
    }

    pub fn ref_has_trait(self, entity: EntityRef) -> bool {
        entity
            .get::<&Traits>()
            .is_some_and(|traits| traits.0.contains(&self))
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

    pub fn take_damage(&mut self, mut damage: f32, entity_ref: EntityRef) {
        if Trait::Fragile.ref_has_trait(entity_ref) {
            damage *= 1.33;
        }
        let endurance = entity_ref
            .get::<&Stats>()
            .map_or(1, |stats| stats.endurance);
        let was_badly_hurt = self.is_badly_hurt();

        self.value -= damage / f32::from(6 + endurance * 3);

        if self.is_badly_hurt()
            && let Some(mut morale) = entity_ref.get::<&mut Morale>()
        {
            if was_badly_hurt {
                morale.apply_negative_effect(Morale::SMALL_INTENSITY, Morale::MEDIUM_DEPTH);
            } else {
                morale.apply_negative_effect(Morale::MEDIUM_INTENSITY, Morale::MEDIUM_DEPTH);
            }
        }
    }

    pub fn restore_fraction(&mut self, fraction: f32, entity_ref: EntityRef) {
        let was_badly_hurt = self.is_badly_hurt();

        self.value = f32::min(1., self.value + fraction);

        if was_badly_hurt
            && !self.is_badly_hurt()
            && let Some(mut morale) = entity_ref.get::<&mut Morale>()
        {
            morale.apply_positive_effect(Morale::SMALL_INTENSITY, Morale::MEDIUM_DEPTH);
        }
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
        let max = 6 + stats.endurance * 3;
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

    pub fn on_move(&mut self) {
        self.dodge_stamina -= 1;
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Morale {
    positive_value: f32,
    negative_value: f32,
}

impl Morale {
    pub const SMALL_INTENSITY: f32 = 1.;
    pub const MEDIUM_INTENSITY: f32 = 3.;
    pub const HIGH_INTENSITY: f32 = 8.;

    pub const SHALLOW_DEPTH: f32 = 2.;
    pub const MEDIUM_DEPTH: f32 = 5.;
    pub const DEEP_DEPTH: f32 = 10.;

    pub fn value(&self) -> f32 {
        self.positive_value - self.negative_value
    }

    fn dodge_mod(&self) -> i16 {
        match self.value() {
            ..-12.0 => -3,
            -12.0..-7.0 => -2,
            -7.0..-3.0 => -1,
            5.0..10.0 => 1,
            10.0.. => 2,
            _ => 0,
        }
    }

    pub fn damage_factor(&self) -> f32 {
        match self.value() {
            ..-10.0 => 0.8,
            -10.0..-5.0 => 0.9,
            _ => 1.,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.value() {
            ..-10.0 => "Devestating",
            -10.0..-5.0 => "Poor",
            5.0..10.0 => "Good",
            10.0.. => "Excellent",
            _ => "Neutral",
        }
    }

    pub fn dampen(&mut self, factor: f32) {
        assert!((0.0..=1.0).contains(&factor));
        self.positive_value *= factor;
        self.negative_value *= factor;
        println!(
            "Bumped values down to PV: {}, NV: {}",
            self.positive_value, self.negative_value
        );
    }

    pub fn apply_positive_effect(&mut self, intensity: f32, depth: f32) {
        assert!(depth >= 1.);
        let prev_value = self.positive_value;
        if self.positive_value < (intensity * depth) {
            let diminishing_factor = 1. - self.positive_value / (intensity * depth);
            self.positive_value += intensity * diminishing_factor;
        }
        println!("Bumped up PV: {} -> {}", prev_value, self.positive_value);
    }
    pub fn apply_negative_effect(&mut self, intensity: f32, depth: f32) {
        assert!(depth >= 1.);
        let prev_value = self.negative_value;
        if self.negative_value < (intensity * depth) {
            let diminishing_factor = 1. - self.negative_value / (intensity * depth);
            self.negative_value += intensity * diminishing_factor;
        }
        println!("Bumped up NV: {} -> {}", prev_value, self.negative_value);
    }

    pub fn journey_start_effect(&mut self) {
        self.apply_positive_effect(Morale::HIGH_INTENSITY, Morale::SHALLOW_DEPTH)
    }
    pub fn new_crew_member_effect(&mut self) {
        self.apply_positive_effect(Morale::MEDIUM_INTENSITY, Morale::SHALLOW_DEPTH)
    }
    pub fn crew_death_effect(&mut self) {
        self.apply_negative_effect(Morale::HIGH_INTENSITY, Morale::DEEP_DEPTH)
    }
}

/// Assumes that any non-ship entities have been despawned.
pub(crate) fn apply_morale_effects_from_crew_state(
    world: &mut World,
    rations_before_eating: usize,
) {
    let mut crew_positive_effect = 0.;
    let mut crew_negative_effect = 0.;

    let crew_count = world
        .query_mut::<()>()
        .with::<&CrewMember>()
        .into_iter()
        .count();
    let crew_character_count = world
        .query_mut::<()>()
        .with::<(&CrewMember, &Character)>()
        .into_iter()
        .count();
    if crew_count == 1 {
        crew_negative_effect += Morale::SMALL_INTENSITY;
    } else if crew_count >= 3 {
        crew_positive_effect += Morale::MEDIUM_INTENSITY;
    }

    let rations_after_eating = world
        .query::<&ItemType>()
        .iter()
        .filter(|&(_, item_type)| *item_type == ItemType::FoodRation)
        .count();
    if rations_before_eating == 0 {
        crew_negative_effect += Morale::SMALL_INTENSITY;
    }
    if rations_after_eating > crew_count {
        crew_positive_effect += Morale::SMALL_INTENSITY;
    }

    let mut weapon_values = world
        .query::<&ItemType>()
        .iter()
        .filter_map(|(_, item_type)| item_type.weapon_properties())
        .map(|weapon_properties| {
            (weapon_properties.damage_mod - 1.
                + if weapon_properties.stun_attack {
                    1.
                } else {
                    0.
                })
            .max(0.)
        })
        .collect::<Vec<_>>();
    weapon_values.sort_by(|a, b| b.total_cmp(a));
    let average_usable_weapon_value =
        weapon_values.iter().take(crew_character_count).sum::<f32>() / crew_character_count as f32;
    match average_usable_weapon_value {
        2.0..3.0 => crew_positive_effect += Morale::SMALL_INTENSITY,
        3.0.. => crew_positive_effect += Morale::MEDIUM_INTENSITY,
        _ => {}
    }

    let fuel_can_count = world
        .query::<&ItemType>()
        .iter()
        .filter(|&(_, item_type)| *item_type == ItemType::FuelCan)
        .count();
    if fuel_can_count >= 1 {
        crew_positive_effect += Morale::SMALL_INTENSITY;
    }

    let medkit_count = world
        .query::<&ItemType>()
        .iter()
        .filter(|&(_, item_type)| *item_type == ItemType::Medkit)
        .count();
    if medkit_count >= 1 {
        crew_positive_effect += Morale::SMALL_INTENSITY;
    }

    for (_, (morale, health, memory)) in world
        .query_mut::<(&mut Morale, Option<&Health>, Option<&CrewLossMemory>)>()
        .with::<&CrewMember>()
    {
        let mut character_negative_effect = crew_negative_effect;
        if let Some(memory) = memory {
            character_negative_effect += if memory.recent {
                Morale::MEDIUM_INTENSITY
            } else {
                Morale::SMALL_INTENSITY
            };
        }
        if health.is_some_and(|health| health.is_badly_hurt()) {
            character_negative_effect += Morale::MEDIUM_INTENSITY;
        }

        morale.apply_positive_effect(crew_positive_effect, Morale::MEDIUM_DEPTH);
        morale.apply_negative_effect(character_negative_effect, Morale::MEDIUM_DEPTH);
    }
}

#[derive(Serialize, Deserialize)]
pub struct SeenWithLowHealth;

pub(crate) fn detect_low_health(
    world: &mut World,
    view_buffer: &mut view::Buffer,
    character: Entity,
) {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let mut command_buffer = CommandBuffer::new();
    for (entity, (pos, health)) in world.query::<(&Pos, &Health)>().iter() {
        let entity_ref = world.entity(entity).unwrap();
        let has_tag = entity_ref.has::<SeenWithLowHealth>();
        let visible_low_health = pos.is_in(area) && health.is_badly_hurt();
        if has_tag && !visible_low_health {
            command_buffer.remove_one::<SeenWithLowHealth>(entity);
        }
        if !has_tag && visible_low_health && health.is_alive() {
            command_buffer.insert_one(entity, SeenWithLowHealth);
            if entity != character {
                let the_entity =
                    NameWithAttribute::lookup_by_ref(entity_ref, view_buffer.assets).definite();
                view_buffer.messages.add(
                    match entity_ref
                        .get::<&Species>()
                        .and_then(|species| species.badly_hurt_behavior())
                    {
                        Some(BadlyHurtBehavior::Fearful) => {
                            format!("{the_entity} is badly hurt, and turns to flee.")
                        }
                        Some(BadlyHurtBehavior::Determined) => {
                            format!(
                                "{the_entity} is badly hurt, and readies themselves to go all-out."
                            )
                        }
                        None => format!("{the_entity} is badly hurt."),
                    },
                );
            }
        }
    }
    command_buffer.run_on(world);
}

#[derive(Serialize, Deserialize)]
pub struct SeenWithLowStamina;

pub(crate) fn detect_low_stamina(
    world: &mut World,
    view_buffer: &mut view::Buffer,
    character: Entity,
) {
    let area = world.get::<&Pos>(character).unwrap().get_area();
    let mut command_buffer = CommandBuffer::new();
    for (entity, (pos, stamina, health)) in world.query::<(&Pos, &Stamina, &Health)>().iter() {
        let entity_ref = world.entity(entity).unwrap();
        let has_tag = entity_ref.has::<SeenWithLowStamina>();
        let visible_low_stamina = pos.is_in(area) && stamina.as_fraction() < 0.6;
        if has_tag && !visible_low_stamina {
            command_buffer.remove_one::<SeenWithLowStamina>(entity);
        }
        if !has_tag && visible_low_stamina && health.is_alive() {
            command_buffer.insert_one(entity, SeenWithLowStamina);
            view_buffer.messages.add(format!(
                "{the_entity} is growing exhausted from dodging attacks.",
                the_entity =
                    NameWithAttribute::lookup_by_ref(entity_ref, view_buffer.assets).definite()
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
