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

    #[derive(Clone, Serialize, Deserialize)]
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

        pub fn load<T, E>(
            &self,
            loader: &mut impl TextureLoader<T, E>,
        ) -> Result<TextureLayer<T>, E> {
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
}

use serde::de::DeserializeOwned;
use std::fmt::Display;
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

pub trait TextureLoader<T, E> {
    fn load_texture(&mut self, name: String) -> Result<T, E>;
}

pub(crate) fn load_json_simple<T: DeserializeOwned>(path: impl Display) -> Result<T, String> {
    let file = File::open(format!("assets/{path}"))
        .map_err(|error| format!("Failed to open file: {error}"))?;
    serde_json::from_reader(file).map_err(|error| format!("Failed to parse file: {error}"))
}
