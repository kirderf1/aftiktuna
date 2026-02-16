pub mod background;
pub mod location;
pub mod model;
pub mod placement;
pub mod profile;
pub mod species;

pub mod color {
    use super::Error;
    use crate::core::SpeciesId;
    use crate::core::display::SpeciesColorId;
    use crate::core::name::Adjective;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

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

    pub fn colors_path(species: SpeciesId) -> impl AsRef<Path> {
        format!("assets/species_color/{species}.json")
    }

    pub fn load_species_color_data(
        species: SpeciesId,
    ) -> Result<HashMap<SpeciesColorId, SpeciesColorEntry>, Error> {
        super::load_from_json(colors_path(species))
    }

    pub struct SpeciesColorMap(HashMap<SpeciesId, HashMap<SpeciesColorId, SpeciesColorEntry>>);

    impl SpeciesColorMap {
        pub fn load() -> Result<Self, Error> {
            let mut map = HashMap::new();
            for entry in fs::read_dir("assets/species_color")
                .map_err(|error| Error::IO(PathBuf::from("assets/species_color"), error))?
            {
                if let Ok(entry) = entry
                    && let Ok(file_name) = entry.file_name().into_string()
                    && let [file_name, "json"] = file_name.split('.').collect::<Vec<_>>()[..]
                {
                    let species_id = SpeciesId::from(file_name);
                    let species_colors = super::load_from_json(entry.path())?;
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

pub(crate) mod dialogue {
    use crate::core::behavior::{self, CrewLossMemory};
    use crate::core::display::DialogueExpression;
    use crate::core::name::Name;
    use crate::core::position::Pos;
    use crate::core::status::{Health, Morale, MoraleState};
    use crate::core::{area, inventory};
    use crate::game_loop::GameState;
    use hecs::Entity;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ConditionedDialogueNode {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub is_badly_hurt: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub is_target_badly_hurt: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub has_enough_fuel: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub is_at_ship: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub is_at_fortuna: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub known_name: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub has_crew_loss_memory: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub has_recent_crew_loss_memory: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub has_background: Option<behavior::BackgroundId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub morale_is_at_least: Option<MoraleState>,
        pub expression: DialogueExpression,
        pub message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub reply: Option<DialogueList>,
    }

    impl ConditionedDialogueNode {
        pub fn is_available(&self, speaker: Entity, target: Entity, state: &GameState) -> bool {
            let world = &state.world;
            self.is_badly_hurt.is_none_or(|is_badly_hurt| {
                is_badly_hurt
                    == world
                        .get::<&Health>(speaker)
                        .is_ok_and(|health| health.is_badly_hurt())
            }) && self
                .is_target_badly_hurt
                .is_none_or(|is_target_badly_hurt| {
                    is_target_badly_hurt
                        == world
                            .get::<&Health>(target)
                            .is_ok_and(|health| health.is_badly_hurt())
                })
                && self.has_enough_fuel.is_none_or(|has_enough_fuel| {
                    has_enough_fuel
                        == area::fuel_needed_to_launch(world).is_some_and(|fuel_amount| {
                            fuel_amount <= inventory::fuel_cans_held_by_crew(world, &[])
                        })
                })
                && self.is_at_ship.is_none_or(|is_at_ship| {
                    is_at_ship
                        == world
                            .get::<&Pos>(speaker)
                            .is_ok_and(|pos| area::is_in_ship(*pos, world))
                })
                && self.is_at_fortuna.is_none_or(|is_at_fortuna| {
                    is_at_fortuna == state.generation_state.is_at_fortuna()
                })
                && self.known_name.is_none_or(|known_name| {
                    Some(known_name) == world.get::<&Name>(speaker).ok().map(|name| name.is_known)
                })
                && self
                    .has_crew_loss_memory
                    .is_none_or(|has_crew_loss_memory| {
                        has_crew_loss_memory == world.satisfies::<&CrewLossMemory>(speaker).unwrap()
                    })
                && self
                    .has_recent_crew_loss_memory
                    .is_none_or(|has_recent_crew_loss_memory| {
                        has_recent_crew_loss_memory
                            == world
                                .get::<&CrewLossMemory>(speaker)
                                .is_ok_and(|crew_loss_memory| crew_loss_memory.recent)
                    })
                && self
                    .has_background
                    .as_ref()
                    .is_none_or(|expected_background| {
                        world.get::<&behavior::BackgroundId>(speaker).is_ok_and(
                            |checked_background| *checked_background == *expected_background,
                        )
                    })
                && self.morale_is_at_least.is_none_or(|morale_state| {
                    morale_state
                        <= world
                            .get::<&Morale>(speaker)
                            .map(|morale| morale.state())
                            .unwrap_or_default()
                })
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct DialogueList(Vec<ConditionedDialogueNode>);

    impl DialogueList {
        pub fn select_node(
            &self,
            speaker: Entity,
            target: Entity,
            state: &GameState,
        ) -> Option<&ConditionedDialogueNode> {
            self.0
                .iter()
                .find(|node| node.is_available(speaker, target, state))
        }
    }

    pub fn load_dialogue_data(name: &str) -> Result<DialogueList, super::Error> {
        super::load_json_asset(format!("dialogue/{name}.json"))
    }
}

pub mod loot {
    use crate::core::item::ItemTypeId;
    use rand::Rng;
    use rand::distr::weighted::WeightedIndex;
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct LootTableId(pub String);

    impl LootTableId {
        pub fn path(&self) -> String {
            format!("loot_table/{}.json", self.0)
        }
    }

    #[derive(Debug, Deserialize)]
    struct LootEntry {
        item: ItemTypeId,
        weight: u16,
    }

    pub(crate) struct LootTable {
        entries: Vec<LootEntry>,
        index_distribution: WeightedIndex<u16>,
    }

    impl LootTable {
        fn load(id: &LootTableId) -> Result<Self, String> {
            let entries: Vec<LootEntry> =
                super::load_json_asset(id.path()).map_err(|error| error.to_string())?;
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
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
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

/// Loads json data from a path relative to the assets directory.
pub(crate) fn load_json_asset<T: DeserializeOwned>(path: impl Display) -> Result<T, Error> {
    load_from_json(format!("assets/{path}"))
}

/// Loads json data from a direct path.
pub(crate) fn load_from_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, Error> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|error| Error::IO(path.to_owned(), error))?;
    let object =
        serde_json::from_reader(file).map_err(|error| Error::Json(path.to_owned(), error))?;
    Ok(object)
}

pub(crate) fn load_aftik_color_names() -> Result<HashMap<SpeciesColorId, Vec<String>>, Error> {
    load_json_asset("selectable_aftik_color_names.json")
}

#[derive(Debug, Deserialize)]
pub(crate) struct CrewData {
    pub points: i32,
    pub crew: Vec<profile::ProfileOrRandom>,
}

impl CrewData {
    pub(crate) fn load_starting_crew() -> Result<CrewData, Error> {
        load_json_asset("starting_crew.json")
    }
}

pub(crate) struct NounDataMap {
    map: HashMap<NounId, NounData>,
    fallback: NounData,
}

impl NounDataMap {
    pub(crate) fn load() -> Result<Self, Error> {
        load_json_asset::<HashMap<NounId, NounData>>("noun_data.json").map(|map| NounDataMap {
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

pub fn load_item_type_map() -> Result<HashMap<ItemTypeId, ItemTypeData>, Error> {
    load_json_asset::<HashMap<ItemTypeId, ItemTypeData>>("item_types.json")
}

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
            item_type_map: load_item_type_map()?,
        })
    }
}
