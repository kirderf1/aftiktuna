pub mod color {
    use super::Error;
    use crate::core::display::AftikColorId;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs::File;

    pub const DEFAULT_COLOR: AftikColorData = AftikColorData {
        primary_color: RGBColor::new(255, 255, 255),
        secondary_color: RGBColor::new(0, 0, 0),
    };

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct AftikColorData {
        pub primary_color: RGBColor,
        pub secondary_color: RGBColor,
    }

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct RGBColor {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    impl RGBColor {
        pub const fn new(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b }
        }
    }

    pub const AFTIK_COLORS_PATH: &str = "assets/aftik_colors.json";

    pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorData>, Error> {
        let file = File::open(AFTIK_COLORS_PATH)?;
        Ok(serde_json::from_reader::<
            _,
            HashMap<AftikColorId, AftikColorData>,
        >(file)?)
    }

    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ColorSource {
        #[default]
        Uncolored,
        Primary,
        Secondary,
    }

    impl ColorSource {
        pub fn get_color(self, aftik_color_data: &AftikColorData) -> RGBColor {
            match self {
                ColorSource::Uncolored => RGBColor::new(255, 255, 255),
                ColorSource::Primary => aftik_color_data.primary_color,
                ColorSource::Secondary => aftik_color_data.secondary_color,
            }
        }
    }
}

pub mod background;

pub(crate) mod loot {
    use crate::core::item;
    use rand::distributions::WeightedIndex;
    use rand::Rng;
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub(crate) struct LootTableId(String);

    #[derive(Debug, Deserialize)]
    struct LootEntry {
        item: item::Type,
        weight: u16,
    }

    pub(crate) struct LootTable {
        entries: Vec<LootEntry>,
        index_distribution: WeightedIndex<u16>,
    }

    impl LootTable {
        fn load(LootTableId(name): &LootTableId) -> Result<Self, String> {
            let entries: Vec<LootEntry> =
                super::load_json_simple(format!("loot_table/{name}.json"))?;
            let index_distribution = WeightedIndex::new(entries.iter().map(|entry| entry.weight))
                .map_err(|error| error.to_string())?;
            Ok(Self {
                entries,
                index_distribution,
            })
        }

        pub(crate) fn pick_loot_item(&self, rng: &mut impl Rng) -> item::Type {
            self.entries[rng.sample(&self.index_distribution)].item
        }
    }

    #[derive(Default)]
    pub(crate) struct LootTableCache(HashMap<LootTableId, LootTable>);

    impl LootTableCache {
        pub(crate) fn get_or_load(
            &mut self,
            loot_table_id: &LootTableId,
        ) -> Result<&LootTable, String> {
            match self.0.entry(loot_table_id.clone()) {
                HashMapEntry::Occupied(entry) => Ok(entry.into_mut()),
                HashMapEntry::Vacant(entry) => {
                    let loot_table = LootTable::load(loot_table_id)?;
                    Ok(entry.insert(loot_table))
                }
            }
        }
    }
}

pub mod model {
    use super::color::ColorSource;
    use super::TextureLoader;
    use crate::view::area::RenderProperties;
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::path::Path;

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Model<T> {
        pub layers: Vec<TextureLayer<T>>,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub wield_offset: (i16, i16),
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub mounted: bool,
    }

    impl<T> Model<T> {
        pub fn is_displacing(&self) -> bool {
            !self.mounted
        }
    }

    impl Model<String> {
        pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<Model<T>, E> {
            let mut layers = Vec::new();
            for layer in &self.layers {
                layers.push(layer.load(loader)?);
            }
            layers.reverse();
            Ok(Model {
                layers,
                wield_offset: self.wield_offset,
                mounted: self.mounted,
            })
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct TextureLayer<T> {
        pub texture: T,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub color: ColorSource,
        #[serde(flatten)]
        pub positioning: LayerPositioning,
        #[serde(flatten)]
        pub conditions: LayerCondition,
    }

    impl TextureLayer<String> {
        pub fn texture_path(&self) -> String {
            format!("object/{}", self.texture)
        }

        fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<TextureLayer<T>, E> {
            let texture = loader.load_texture(self.texture_path())?;
            Ok(TextureLayer {
                texture,
                color: self.color,
                positioning: self.positioning.clone(),
                conditions: self.conditions.clone(),
            })
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct LayerPositioning {
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub size: Option<(i16, i16)>,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub y_offset: i16,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub fixed: bool,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct LayerCondition {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub if_cut: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub if_alive: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub if_hurt: Option<bool>,
    }

    impl LayerCondition {
        pub fn meets_conditions(&self, properties: &RenderProperties) -> bool {
            (self.if_cut.is_none() || self.if_cut == Some(properties.is_cut))
                && (self.if_alive.is_none() || self.if_alive == Some(properties.is_alive))
                && (self.if_hurt.is_none() || self.if_hurt == Some(properties.is_badly_hurt))
        }
    }

    pub fn load_raw_model_from_path(
        file_path: impl AsRef<Path>,
    ) -> Result<Model<String>, super::Error> {
        let file = File::open(file_path)?;
        Ok(serde_json::from_reader::<_, Model<String>>(file)?)
    }
}

use crate::core::display::AftikColorId;
use crate::core::status::{Stats, Traits};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::File;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(error) => Display::fmt(error, f),
            Error::Json(error) => Display::fmt(error, f),
        }
    }
}

pub trait TextureLoader<T, E> {
    fn load_texture(&mut self, name: String) -> Result<T, E>;
}

pub(crate) fn load_json_simple<T: DeserializeOwned>(path: impl Display) -> Result<T, String> {
    let file = File::open(format!("assets/{path}"))
        .map_err(|error| format!("Failed to open file: {error}"))?;
    serde_json::from_reader(file).map_err(|error| format!("Failed to parse file: {error}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AftikProfile {
    pub name: String,
    pub color: AftikColorId,
    pub stats: Stats,
    #[serde(default)]
    pub traits: Traits,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ProfileOrRandom {
    #[default]
    Random,
    #[serde(untagged)]
    Profile(AftikProfile),
}

impl ProfileOrRandom {
    pub(crate) fn unwrap(
        self,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) -> Option<AftikProfile> {
        match self {
            ProfileOrRandom::Random => remove_random_profile(character_profiles, rng),
            ProfileOrRandom::Profile(profile) => Some(profile),
        }
    }
}

pub(crate) fn remove_random_profile(
    character_profiles: &mut Vec<AftikProfile>,
    rng: &mut impl Rng,
) -> Option<AftikProfile> {
    if character_profiles.is_empty() {
        eprintln!("Tried picking a random profile, but there were none left to choose.");
        return None;
    }
    let chosen_index = rng.gen_range(0..character_profiles.len());
    Some(character_profiles.swap_remove(chosen_index))
}

pub(crate) fn load_character_profiles() -> Result<Vec<AftikProfile>, String> {
    load_json_simple("character_profiles.json")
        .map_err(|message| format!("Problem loading \"character_profiles.json\": {message}"))
}

#[derive(Debug, Deserialize)]
pub(crate) struct CrewData {
    pub points: i32,
    pub crew: Vec<ProfileOrRandom>,
}

impl CrewData {
    pub(crate) fn load_starting_crew() -> Result<CrewData, String> {
        load_json_simple("starting_crew.json")
            .map_err(|message| format!("Problem loading \"starting_crew.json\": {message}"))
    }
}
