pub mod area;
pub mod behavior;
pub(crate) mod combat;
pub(crate) mod inventory;
pub mod item;
pub mod name;
pub mod position;
pub mod status;

pub mod display {
    use crate::asset::{SpeciesData, WeightedVariant};
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
        pub(crate) fn random_for_species(
            species_data: &SpeciesData,
            rng: &mut impl rand::Rng,
        ) -> Self {
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
}

pub mod store {
    use crate::game_loop::GameState;
    use crate::view;
    use hecs::{Entity, Ref, World};
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

    use super::item;

    #[derive(Serialize, Deserialize)]
    pub struct Points(pub i32);

    #[derive(Serialize, Deserialize)]
    pub(crate) struct Shopkeeper(pub Vec<StoreStock>);

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum StockQuantity {
        Unlimited,
        Count(u16),
    }

    impl StockQuantity {
        pub fn is_zero(&self) -> bool {
            matches!(self, Self::Count(0))
        }

        pub fn subtracted(&self, subtracted: u16) -> Option<Self> {
            match self {
                Self::Unlimited => Some(Self::Unlimited),
                Self::Count(count) => Some(Self::Count(count.checked_sub(subtracted)?)),
            }
        }
    }

    impl Display for StockQuantity {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unlimited => "Unlimited".fmt(f),
                Self::Count(0) => "SOLD OUT".fmt(f),
                Self::Count(count) => count.fmt(f),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub(crate) struct StoreStock {
        pub item: item::ItemTypeId,
        pub price: item::Price,
        pub quantity: StockQuantity,
    }

    #[derive(Serialize, Deserialize)]
    pub struct IsTrading(pub Entity);

    pub(crate) fn get_shop_info(world: &World, character: Entity) -> Option<Ref<'_, Shopkeeper>> {
        let shopkeeper = world.get::<&IsTrading>(character).ok()?.0;
        world.get::<&Shopkeeper>(shopkeeper).ok()
    }

    pub(crate) fn initiate_trade(
        character: Entity,
        shopkeeper: Entity,
        state: &mut GameState,
        view_buffer: &mut view::Buffer,
    ) {
        state
            .world
            .insert_one(character, IsTrading(shopkeeper))
            .unwrap();

        view_buffer.add_change_message(
            "\"Welcome to the store. What do you want to buy?\"".to_owned(),
            state,
        );
    }
}

use hecs::Entity;
use serde::{Deserialize, Serialize};

pub const CREW_SIZE_LIMIT: usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesId(String);

impl SpeciesId {
    pub fn is_aftik(&self) -> bool {
        self.0 == "aftik"
    }
    pub fn model_id(&self) -> display::ModelId {
        display::ModelId(format!("creature/{self}"))
    }

    pub fn portrait_model_id(&self) -> display::ModelId {
        display::ModelId(format!("portrait/{self}"))
    }

    pub fn noun_id(&self) -> name::NounId {
        name::NounId(self.0.clone())
    }
}

impl From<&str> for SpeciesId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl std::fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag(String);

pub fn any_alive_with_tag(target_tag: &Tag, world: &hecs::World) -> bool {
    world
        .query::<(&status::Health, &Tag)>()
        .iter()
        .any(|(_, (health, tag))| health.is_alive() && target_tag == tag)
}

pub fn find_one_entity_with_tag(target_tag: &Tag, world: &hecs::World) -> Option<Entity> {
    world
        .query::<&Tag>()
        .iter()
        .find(|&(_, tag)| tag == target_tag)
        .map(|(entity, _)| entity)
}

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
pub struct OpenedChest;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Door {
    pub kind: DoorKind,
    pub destination: position::Pos,
    pub door_pair: Entity,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DoorKind {
    Door,
    Path,
}

#[derive(Serialize, Deserialize)]
pub struct IsCut;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Stuck,
    Sealed,
}

impl BlockType {
    pub fn variants() -> &'static [Self] {
        use BlockType::*;
        &[Stuck, Sealed]
    }

    pub fn description(self) -> &'static str {
        match self {
            BlockType::Stuck => "stuck",
            BlockType::Sealed => "sealed shut",
        }
    }

    pub fn usable_tools(self) -> Vec<item::Tool> {
        match self {
            BlockType::Stuck => vec![item::Tool::Crowbar, item::Tool::Blowtorch],
            BlockType::Sealed => vec![item::Tool::Blowtorch],
        }
    }
}

/// Represents a dialogue asset path, starting from the dialogue directory and without the file ending.
pub type DialogueId = String;
