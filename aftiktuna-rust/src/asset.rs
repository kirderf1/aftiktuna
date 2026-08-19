pub mod background;
pub mod dialogue;
pub mod location;
pub mod model;
pub mod placement;
pub mod profile;
pub mod species;

pub mod color {
    use super::Error;
    use crate::asset::AssetDirectory;
    use crate::core::SpeciesId;
    use crate::core::display::SpeciesColorId;
    use crate::core::name::Adjective;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs;

    pub const DEFAULT_COLOR: SpeciesColorData = SpeciesColorData {
        primary_color: RGBColor::new(255, 255, 255),
        secondary_color: RGBColor::new(0, 0, 0),
    };

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct SpeciesColorData {
        pub primary_color: RGBColor,
        pub secondary_color: RGBColor,
    }

    impl Default for SpeciesColorData {
        fn default() -> Self {
            DEFAULT_COLOR
        }
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

    #[derive(Clone, Default, Serialize, Deserialize)]
    pub struct SpeciesColorEntry {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub adjective: Option<Adjective>,
        #[serde(flatten)]
        pub color_data: SpeciesColorData,
    }

    pub const SPECIES_COLOR_DIR: AssetDirectory<HashMap<SpeciesColorId, SpeciesColorEntry>> =
        AssetDirectory::new("species_color");

    pub struct SpeciesColorMap(HashMap<SpeciesId, HashMap<SpeciesColorId, SpeciesColorEntry>>);

    impl SpeciesColorMap {
        pub fn load() -> Result<Self, Error> {
            let mut map = HashMap::new();
            for entry in fs::read_dir(SPECIES_COLOR_DIR.dir_path())
                .map_err(|error| Error::IO(SPECIES_COLOR_DIR.dir_path(), error))?
            {
                if let Ok(entry) = entry
                    && let Ok(file_name) = entry.file_name().into_string()
                    && let [file_name, "json"] = file_name.split('.').collect::<Vec<_>>()[..]
                {
                    let species_id = SpeciesId::from(file_name);
                    let species_colors = SPECIES_COLOR_DIR.load(file_name)?;
                    map.insert(species_id, species_colors);
                }
            }
            Ok(Self(map))
        }

        pub fn get(
            &self,
            species_id: &SpeciesId,
            color_id: &SpeciesColorId,
        ) -> Option<&SpeciesColorEntry> {
            self.0.get(species_id)?.get(color_id)
        }

        pub fn available_ids(
            &self,
            species_id: &SpeciesId,
        ) -> impl Iterator<Item = &SpeciesColorId> {
            self.0
                .get(species_id)
                .map(HashMap::keys)
                .unwrap_or_default()
        }
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
        pub fn get_color(self, color_data: &SpeciesColorData) -> RGBColor {
            match self {
                ColorSource::Uncolored => RGBColor::new(255, 255, 255),
                ColorSource::Primary => color_data.primary_color,
                ColorSource::Secondary => color_data.secondary_color,
            }
        }
    }
}

pub mod loot {
    use crate::asset::AssetDirectory;
    use crate::core::item::ItemTypeId;
    use rand::Rng;
    use rand::distr::weighted::WeightedIndex;
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct LootTableId(pub String);

    impl std::fmt::Display for LootTableId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct LootEntry {
        item: ItemTypeId,
        weight: u16,
    }

    pub const LOOT_TABLE_DIR: AssetDirectory<Vec<LootEntry>> = AssetDirectory::new("loot_table");

    pub(crate) struct LootTable {
        entries: Vec<LootEntry>,
        index_distribution: WeightedIndex<u16>,
    }

    impl LootTable {
        fn load(id: &LootTableId) -> Result<Self, String> {
            let entries: Vec<LootEntry> =
                LOOT_TABLE_DIR.load(id).map_err(|error| error.to_string())?;
            let index_distribution = WeightedIndex::new(entries.iter().map(|entry| entry.weight))
                .map_err(|error| error.to_string())?;
            Ok(Self {
                entries,
                index_distribution,
            })
        }

        pub(crate) fn pick_loot_item(&self, rng: &mut impl Rng) -> &ItemTypeId {
            &self.entries[rng.sample(&self.index_distribution)].item
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

use crate::core::combat::WeaponProperties;
use crate::core::display::SpeciesColorId;
use crate::core::item::{ItemTypeId, Price};
use crate::core::name::{NounData, NounId};
use crate::core::status::StatChanges;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    IO(PathBuf, std::io::Error),
    Json(PathBuf, serde_json::Error),
    Validation(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(file, error) => write!(
                f,
                "Problem accessing \"{file}\": {error}",
                file = file.display()
            ),
            Error::Json(file, error) => {
                write!(
                    f,
                    "Problem parsing \"{file}\": {error}",
                    file = file.display()
                )
            }
            Error::Validation(error) => {
                write!(f, "Problem validating assets: {error}")
            }
        }
    }
}

pub trait TextureLoader<T, E> {
    fn load_texture(&mut self, name: String) -> Result<T, E>;
}

/// Loads json data from a direct path.
fn load_from_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, Error> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|error| Error::IO(path.to_owned(), error))?;
    let object =
        serde_json::from_reader(file).map_err(|error| Error::Json(path.to_owned(), error))?;
    Ok(object)
}

pub struct AssetFile<T> {
    path: &'static str,
    data_type: PhantomData<T>,
}

impl<T: DeserializeOwned> AssetFile<T> {
    pub(crate) const fn new(path: &'static str) -> Self {
        Self {
            path,
            data_type: PhantomData,
        }
    }
    pub fn matches(&self, path: &Path) -> bool {
        path.ends_with(self.path)
    }
    pub fn file_path(&self) -> PathBuf {
        format!("assets/{}", self.path).into()
    }
    pub fn load(&self) -> Result<T, Error> {
        load_from_json::<T>(format!("assets/{}", self.path))
    }
}

impl<K: Eq + Hash + DeserializeOwned, V: DeserializeOwned> AssetFile<HashMap<K, V>> {
    /// Loads asset as order-preserved map.
    pub fn load_index_map(&self) -> Result<IndexMap<K, V>, Error> {
        load_from_json::<IndexMap<K, V>>(format!("assets/{}", self.path))
    }
}

pub struct AssetDirectory<T> {
    path: &'static str,
    data_type: PhantomData<T>,
}

impl<T: DeserializeOwned> AssetDirectory<T> {
    pub(crate) const fn new(path: &'static str) -> Self {
        Self {
            path,
            data_type: PhantomData,
        }
    }
    pub fn dir_path(&self) -> PathBuf {
        format!("assets/{}", self.path).into()
    }
    pub fn file_path(&self, id: impl Display) -> PathBuf {
        format!("assets/{}/{id}.json", self.path).into()
    }
    pub fn load(&self, id: impl Display) -> Result<T, Error> {
        load_from_json::<T>(format!("assets/{}/{id}.json", self.path))
    }
}

impl<K: Eq + Hash + DeserializeOwned, V: DeserializeOwned> AssetDirectory<HashMap<K, V>> {
    /// Loads asset as order-preserved map.
    pub fn load_index_map(&self, id: impl Display) -> Result<IndexMap<K, V>, Error> {
        load_from_json::<IndexMap<K, V>>(format!("assets/{}/{id}.json", self.path))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CrewData {
    pub points: i32,
    pub crew: Vec<profile::ProfileOrRandom>,
}

pub(crate) struct NounDataMap {
    map: HashMap<NounId, NounData>,
    fallback: NounData,
}

impl NounDataMap {
    pub(crate) fn load() -> Result<Self, Error> {
        NOUN_DATA_FILE.load().map(|map| NounDataMap {
            map,
            fallback: NounData::default(),
        })
    }

    pub(crate) fn lookup(&self, noun_id: &NounId) -> &NounData {
        self.map.get(noun_id).unwrap_or(&self.fallback)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ItemUseType {
    Medkit {
        restore_fraction: f32,
        use_duration: u16,
    },
    BlackOrb {
        change: StatChanges,
    },
    OddHandMirror {
        sum_change: i16,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemTypeData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) weapon: Option<WeaponProperties>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) usage: Option<ItemUseType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) price: Option<Price>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) extra_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) shop_description: Option<String>,
}

impl ItemTypeData {
    pub fn is_medkit(&self) -> bool {
        matches!(self.usage, Some(ItemUseType::Medkit { .. }))
    }
}

pub const CHARACTER_NAMES_FILE: AssetFile<Vec<String>> = AssetFile::new("character_names.json");
pub const AFTIK_COLOR_NAMES_FILE: AssetFile<HashMap<SpeciesColorId, Vec<String>>> =
    AssetFile::new("selectable_aftik_color_names.json");
pub(crate) const CREW_DATA_FILE: AssetFile<CrewData> = AssetFile::new("starting_crew.json");
pub const NOUN_DATA_FILE: AssetFile<HashMap<NounId, NounData>> = AssetFile::new("noun_data.json");
pub const ITEM_TYPES_FILE: AssetFile<HashMap<ItemTypeId, ItemTypeData>> =
    AssetFile::new("item_types.json");

pub struct GameAssets {
    pub(crate) noun_data_map: NounDataMap,
    pub(crate) species_data_map: species::SpeciesDataMap,
    pub(crate) color_map: color::SpeciesColorMap,
    pub(crate) item_type_map: HashMap<ItemTypeId, ItemTypeData>,
}

impl GameAssets {
    pub fn load() -> Result<Self, Error> {
        Ok(Self {
            noun_data_map: NounDataMap::load()?,
            species_data_map: species::load_species_map()?,
            color_map: color::SpeciesColorMap::load()?,
            item_type_map: ITEM_TYPES_FILE.load()?,
        })
    }
}
