use crate::asset::color::SpeciesColorMap;
use crate::core::SpeciesId;
use crate::core::display::SpeciesColorId;
use crate::core::status::{Stats, Traits};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StatsOrRandom {
    Random {
        #[serde(default, skip_serializing_if = "crate::is_default")]
        stats_bonus: i16,
    },
    #[serde(untagged)]
    Stats(Stats),
}

impl StatsOrRandom {
    pub(crate) fn unwrap_or_else(self, random_selection: impl FnOnce(i16) -> Stats) -> Stats {
        match self {
            Self::Random { stats_bonus } => random_selection(stats_bonus),
            Self::Stats(stats) => stats,
        }
    }
}

impl Default for StatsOrRandom {
    fn default() -> Self {
        Self::Random { stats_bonus: 0 }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraitsOrRandom {
    #[default]
    Random,
    #[serde(untagged)]
    Traits(Traits),
}

impl TraitsOrRandom {
    pub(crate) fn unwrap_or_else(self, random_selection: impl FnOnce() -> Traits) -> Traits {
        match self {
            Self::Random => random_selection(),
            Self::Traits(traits) => traits,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub species: SpeciesId,
    pub name: String,
    pub color: SpeciesColorId,
    pub stats: StatsOrRandom,
    pub traits: TraitsOrRandom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileOrRandom {
    Random {
        species: SpeciesId,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        stats_bonus: i16,
    },
    #[serde(untagged)]
    Profile(CharacterProfile),
}

impl ProfileOrRandom {
    pub(crate) fn unwrap<'a>(
        self,
        character_names: &mut Vec<String>,
        aftik_color_names: &mut HashMap<SpeciesColorId, Vec<String>>,
        color_map: &SpeciesColorMap,
        rng: &mut impl Rng,
        query_used_colors: impl FnOnce(&SpeciesId) -> Vec<&'a SpeciesColorId>,
    ) -> Option<CharacterProfile> {
        match self {
            ProfileOrRandom::Random {
                species,
                stats_bonus,
            } => {
                let used_colors = query_used_colors(&species);
                random_profile(
                    species,
                    StatsOrRandom::Random { stats_bonus },
                    &used_colors,
                    character_names,
                    aftik_color_names,
                    color_map,
                    rng,
                )
            }
            ProfileOrRandom::Profile(profile) => Some(profile),
        }
    }
}

pub(crate) fn random_profile(
    species_id: SpeciesId,
    stats: StatsOrRandom,
    used_colors: &[&SpeciesColorId],
    character_names: &mut Vec<String>,
    aftik_color_names: &mut HashMap<SpeciesColorId, Vec<String>>,
    color_map: &SpeciesColorMap,
    rng: &mut impl Rng,
) -> Option<CharacterProfile> {
    let (name, color) = if species_id.is_aftik() {
        random_aftik_profile(aftik_color_names, rng, used_colors)?
    } else {
        use rand::seq::IteratorRandom;
        let chosen_color = color_map
            .available_ids(&species_id)
            .filter(|color| !used_colors.contains(color))
            .choose_stable(rng)
            .cloned();
        let Some(chosen_color) = chosen_color else {
            eprintln!("Tried picking a random color, but there were none left to choose.");
            return None;
        };
        if character_names.is_empty() {
            eprintln!("Tried picking a random name, but there were none left to choose.");
            return None;
        }
        let chosen_name = character_names.swap_remove(rng.random_range(0..character_names.len()));
        (chosen_name, chosen_color)
    };
    Some(CharacterProfile {
        species: species_id,
        name,
        color,
        stats,
        traits: TraitsOrRandom::Random,
    })
}

pub(crate) fn random_aftik_profile(
    aftik_color_names: &mut HashMap<SpeciesColorId, Vec<String>>,
    rng: &mut impl Rng,
    used_aftik_colors: &[&SpeciesColorId],
) -> Option<(String, SpeciesColorId)> {
    use rand::seq::IteratorRandom;
    let chosen_color = aftik_color_names
        .iter()
        .filter(|(color, names)| !used_aftik_colors.contains(color) && !names.is_empty())
        .map(|(color, _)| color)
        .choose_stable(rng)
        .cloned();
    let Some(chosen_color) = chosen_color else {
        eprintln!("Tried picking a random name and color, but there were none left to choose.");
        return None;
    };
    let name_choices = aftik_color_names.get_mut(&chosen_color).unwrap();
    let chosen_name = name_choices.swap_remove(rng.random_range(0..name_choices.len()));
    Some((chosen_name, chosen_color))
}
