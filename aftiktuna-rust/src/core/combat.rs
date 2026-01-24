use super::item::ItemTypeId;
use super::{Species, inventory};
use crate::asset::GameAssets;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackKind {
    Light,
    Rash,
    Charged,
}

impl AttackKind {
    /// Value modifier to the d20 roll for the attack hit accuracy.
    pub fn hit_modifier(self) -> i16 {
        match self {
            Self::Light => 0,
            Self::Rash => -2,
            Self::Charged => 2,
        }
    }

    /// Value modifier to the d20 roll to apply the stun effect.
    pub fn stun_modifier(self) -> i16 {
        match self {
            Self::Light => -3,
            Self::Rash => 3,
            Self::Charged => 6,
        }
    }

    pub fn damage_modifier(self) -> f32 {
        match self {
            Self::Light => 1.,
            Self::Rash => 1.75,
            Self::Charged => 2.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnarmedType {
    Bite,
    Scratch,
    Punch,
    Pounce,
    Slash,
}

impl UnarmedType {
    pub fn attack_verb(self) -> &'static str {
        match self {
            Self::Bite | Self::Pounce => "jumps at",
            Self::Scratch => "scratches at",
            Self::Punch => "launches a punch at",
            Self::Slash => "slashes at",
        }
    }

    pub fn hit_verb(self) -> &'static str {
        match self {
            Self::Bite => "bites",
            Self::Scratch | Self::Punch | Self::Slash => "hits",
            Self::Pounce => "pounces",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackSet {
    Light,
    Quick,
    Slow,
    Intense,
    Varied,
}

impl AttackSet {
    pub fn available_kinds(self) -> &'static [AttackKind] {
        use AttackKind::*;
        match self {
            AttackSet::Light => &[Light],
            AttackSet::Quick => &[Light, Rash],
            AttackSet::Slow => &[Light, Charged],
            AttackSet::Intense => &[Rash, Charged],
            AttackSet::Varied => &[Light, Rash, Charged],
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WeaponProperties {
    pub damage_mod: f32,
    pub attack_set: AttackSet,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub stun_attack: bool,
}

impl Default for WeaponProperties {
    fn default() -> Self {
        Self {
            damage_mod: 2.,
            attack_set: AttackSet::Varied,
            stun_attack: false,
        }
    }
}

pub fn get_active_weapon_properties(
    world: &hecs::World,
    attacker: hecs::Entity,
    assets: &GameAssets,
) -> WeaponProperties {
    inventory::get_wielded(world, attacker)
        .and_then(|item| {
            world
                .get::<&ItemTypeId>(item)
                .ok()
                .and_then(|item_type| assets.item_type_map.get(&item_type))
                .and_then(|data| data.weapon)
        })
        .or_else(|| {
            assets
                .species_data_map
                .get(&world.get::<&Species>(attacker).unwrap())
                .map(|species_data| species_data.unarmed_properties())
        })
        .unwrap_or_default()
}
