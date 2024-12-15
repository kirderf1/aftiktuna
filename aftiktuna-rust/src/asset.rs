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

pub mod background {
    use super::TextureLoader;
    use crate::core::area::BackgroundId;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs::File;

    #[derive(Serialize, Deserialize)]
    pub struct RawBGData {
        #[serde(flatten)]
        pub primary: RawPrimaryBGData,
        #[serde(flatten)]
        pub portrait: PortraitBGData<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(from = "ParallaxLayerOrList", into = "ParallaxLayerOrList")]
    pub struct RawPrimaryBGData(pub Parallax<String>);

    #[derive(Serialize, Deserialize)]
    pub enum PortraitBGData<T> {
        #[serde(rename = "portrait_color")]
        Color([u8; 3]),
        #[serde(rename = "portrait_texture")]
        Texture(T),
    }

    impl PortraitBGData<String> {
        pub fn load<T, E>(
            &self,
            loader: &mut impl TextureLoader<T, E>,
        ) -> Result<PortraitBGData<T>, E> {
            Ok(match self {
                PortraitBGData::Color(color) => PortraitBGData::Color(*color),
                PortraitBGData::Texture(texture) => {
                    PortraitBGData::Texture(load_texture(texture, loader)?)
                }
            })
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    enum ParallaxLayerOrList {
        Layer(ParallaxLayer<String>),
        Parallax(Parallax<String>),
    }

    impl From<RawPrimaryBGData> for ParallaxLayerOrList {
        fn from(RawPrimaryBGData(parallax): RawPrimaryBGData) -> Self {
            if parallax.layers.len() != 1 {
                Self::Parallax(parallax)
            } else {
                Self::Layer(parallax.layers.into_iter().next().unwrap())
            }
        }
    }

    impl From<ParallaxLayerOrList> for RawPrimaryBGData {
        fn from(value: ParallaxLayerOrList) -> Self {
            Self(match value {
                ParallaxLayerOrList::Layer(layer) => Parallax {
                    layers: vec![layer],
                },
                ParallaxLayerOrList::Parallax(parallax) => parallax,
            })
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Parallax<T> {
        pub layers: Vec<ParallaxLayer<T>>,
    }

    impl Parallax<String> {
        pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<Parallax<T>, E> {
            Ok(Parallax {
                layers: self
                    .layers
                    .iter()
                    .map(|layer| layer.load(loader))
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ParallaxLayer<T> {
        pub texture: T,
        #[serde(default = "default_move_factor")]
        pub move_factor: f32,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub is_looping: bool,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub offset: Offset,
    }

    fn default_move_factor() -> f32 {
        1.
    }

    impl ParallaxLayer<String> {
        pub fn load<T, E>(
            &self,
            loader: &mut impl TextureLoader<T, E>,
        ) -> Result<ParallaxLayer<T>, E> {
            Ok(ParallaxLayer {
                texture: load_texture(&self.texture, loader)?,
                move_factor: self.move_factor,
                is_looping: self.is_looping,
                offset: self.offset,
            })
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Offset {
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub x: i16,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub y: i16,
    }

    pub const DATA_FILE_PATH: &str = "assets/texture/background/backgrounds.json";

    pub fn load_raw_backgrounds() -> Result<HashMap<BackgroundId, RawBGData>, super::Error> {
        let file = File::open(DATA_FILE_PATH)?;
        Ok(serde_json::from_reader::<
            _,
            HashMap<BackgroundId, RawBGData>,
        >(file)?)
    }

    pub fn load_index_map_backgrounds() -> Result<IndexMap<BackgroundId, RawBGData>, super::Error> {
        let file = File::open(DATA_FILE_PATH)?;
        Ok(serde_json::from_reader::<
            _,
            IndexMap<BackgroundId, RawBGData>,
        >(file)?)
    }

    fn load_texture<T, E>(texture: &str, loader: &mut impl TextureLoader<T, E>) -> Result<T, E> {
        loader.load_texture(format!("background/{texture}"))
    }
}

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
