use crate::asset::AssetFile;
use crate::core::SpeciesId;
use crate::core::behavior::BadlyHurtBehavior;
use crate::core::combat::{AttackSet, UnarmedType, WeaponProperties};
use crate::core::display::CreatureVariant;
use crate::core::status::Stats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct WeightedVariant {
    pub variant: CreatureVariant,
    pub weight: u16,
}

#[derive(Debug)]
pub enum SpeciesKind {
    CharacterSpecies,
    Fauna {
        agressive_by_default: bool,
        tameable: bool,
    },
}

#[derive(Debug)]
pub struct SpeciesData {
    pub kind: SpeciesKind,
    pub default_stats: Stats,
    pub is_large: bool,
    pub unarmed: UnarmedType,
    pub attack_set: AttackSet,
    pub badly_hurt_behavior: Option<BadlyHurtBehavior>,
    pub variant_groups: Vec<Vec<WeightedVariant>>,
}

impl SpeciesData {
    pub fn unarmed_properties(&self) -> WeaponProperties {
        WeaponProperties {
            damage_mod: 2.0,
            attack_set: self.attack_set,
            stun_attack: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CharacterSpeciesData {
    default_stats: Stats,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    is_large: bool,
    unarmed: UnarmedType,
    attack_set: AttackSet,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    badly_hurt_behavior: Option<BadlyHurtBehavior>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    variant_groups: Vec<Vec<WeightedVariant>>,
}

impl From<CharacterSpeciesData> for SpeciesData {
    fn from(value: CharacterSpeciesData) -> Self {
        let CharacterSpeciesData {
            default_stats,
            is_large,
            unarmed,
            attack_set,
            badly_hurt_behavior,
            variant_groups,
        } = value;
        Self {
            kind: SpeciesKind::CharacterSpecies,
            default_stats,
            is_large,
            unarmed,
            attack_set,
            badly_hurt_behavior,
            variant_groups,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FaunaData {
    default_stats: Stats,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    is_large: bool,
    unarmed: UnarmedType,
    attack_set: AttackSet,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    badly_hurt_behavior: Option<BadlyHurtBehavior>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    agressive_by_default: bool,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    tameable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    variant_groups: Vec<Vec<WeightedVariant>>,
}

impl From<FaunaData> for SpeciesData {
    fn from(value: FaunaData) -> Self {
        let FaunaData {
            default_stats,
            is_large,
            unarmed,
            attack_set,
            badly_hurt_behavior,
            agressive_by_default,
            tameable,
            variant_groups,
        } = value;
        Self {
            kind: SpeciesKind::Fauna {
                agressive_by_default,
                tameable,
            },
            default_stats,
            is_large,
            unarmed,
            attack_set,
            badly_hurt_behavior,
            variant_groups,
        }
    }
}

pub type SpeciesDataMap = HashMap<SpeciesId, SpeciesData>;

const SPECIES_FILE: AssetFile<HashMap<SpeciesId, CharacterSpeciesData>> =
    AssetFile::new("species.json");

const FAUNA_FILE: AssetFile<HashMap<SpeciesId, FaunaData>> = AssetFile::new("fauna.json");

pub(super) fn load_species_map() -> Result<SpeciesDataMap, super::Error> {
    let character_species_map = SPECIES_FILE.load()?;
    let fauna_map = FAUNA_FILE.load()?;
    let mut species_map = SpeciesDataMap::new();
    for (species, data) in fauna_map {
        species_map.insert(species, data.into());
    }
    for (species_id, data) in character_species_map {
        if species_map
            .insert(species_id.clone(), data.into())
            .is_some()
        {
            return Err(super::Error::Validation(format!(
                "\"{species_id}\" has been defined as both species and fauna."
            )));
        }
    }

    Ok(species_map)
}

/// Loads species.json and discards the data to return just the species ids, excluding fauna.
pub fn load_species_list() -> Result<Vec<SpeciesId>, super::Error> {
    Ok(SPECIES_FILE.load_index_map()?.into_keys().collect())
}

/// Loads fauna.json and discards the data to return just the species ids of fauna only.
pub fn load_fauna_list() -> Result<Vec<SpeciesId>, super::Error> {
    Ok(FAUNA_FILE.load_index_map()?.into_keys().collect())
}
