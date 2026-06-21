use crate::asset::species::{SpeciesData, WeightedVariant};
use rand::distr::Distribution;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn unknown() -> Self {
        Self::new("unknown")
    }
    pub fn small_unknown() -> Self {
        Self::new("small_unknown")
    }

    pub fn fortuna_chest() -> Self {
        Self::new("container/fortuna_chest")
    }

    pub fn ship() -> Self {
        Self::new("ship")
    }

    pub fn ship_controls() -> Self {
        Self::new("ship_controls")
    }

    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn item(name: &str) -> Self {
        Self(format!("item/{name}"))
    }

    pub fn path(&self) -> &str {
        &self.0
    }

    pub fn file_path(&self) -> impl AsRef<Path> + use<> {
        let Self(path) = self;
        format!("assets/texture/object/{path}.json")
    }
}

impl Default for ModelId {
    fn default() -> Self {
        Self::unknown()
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesColorId(pub String);

impl SpeciesColorId {
    pub fn new(name: &str) -> Self {
        SpeciesColorId(name.to_owned())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogueExpression {
    #[default]
    Neutral,
    Excited,
    Sad,
}

impl DialogueExpression {
    pub fn variants() -> &'static [Self] {
        use DialogueExpression::*;
        &[Neutral, Excited, Sad]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CreatureVariant(String);

impl CreatureVariant {
    pub fn female() -> Self {
        Self("female".to_owned())
    }
    pub fn male() -> Self {
        Self("male".to_owned())
    }
}

impl std::fmt::Display for CreatureVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreatureVariantSet(pub HashSet<CreatureVariant>);

impl CreatureVariantSet {
    pub(crate) fn random_for_species(species_data: &SpeciesData, rng: &mut impl rand::Rng) -> Self {
        let mut variant_set = Self::default();
        for variant_group in &species_data.variant_groups {
            if let Some(variant) = pick_creature_variant(variant_group, rng) {
                variant_set.0.insert(variant);
            }
        }
        variant_set
    }

    pub(crate) fn insert_missing_variants(
        &mut self,
        species_data: &SpeciesData,
        rng: &mut impl rand::Rng,
    ) {
        for variant_group in &species_data.variant_groups {
            if variant_group
                .iter()
                .all(|entry| !self.0.contains(&entry.variant))
                && let Some(variant) = pick_creature_variant(variant_group, rng)
            {
                self.0.insert(variant);
            }
        }
    }
}

fn pick_creature_variant(
    variant_group: &[WeightedVariant],
    rng: &mut impl rand::Rng,
) -> Option<CreatureVariant> {
    let weight_distribution = rand::distr::weighted::WeightedIndex::new(
        variant_group.iter().map(|variant| variant.weight),
    )
    .ok()?;
    Some(
        variant_group[weight_distribution.sample(rng)]
            .variant
            .clone(),
    )
}

impl<T: IntoIterator<Item = CreatureVariant>> From<T> for CreatureVariantSet {
    fn from(value: T) -> Self {
        Self(value.into_iter().collect())
    }
}
