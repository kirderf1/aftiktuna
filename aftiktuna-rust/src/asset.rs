pub mod background;
pub mod location;
pub mod model;

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

pub mod loot {
    use crate::core::item;
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
        item: item::Type,
        weight: u16,
    }

    pub(crate) struct LootTable {
        entries: Vec<LootEntry>,
        index_distribution: WeightedIndex<u16>,
    }

    impl LootTable {
        fn load(id: &LootTableId) -> Result<Self, String> {
            let entries: Vec<LootEntry> = super::load_json_simple(id.path())?;
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

pub mod placement {
    use crate::asset::model::{Model, ModelAccess, Offsets};
    use crate::core::position::Coord;
    use crate::view::area::ObjectRenderData;
    use std::collections::HashMap;
    use std::mem;

    pub type Vec2 = (f32, f32);

    // Coordinates are mapped like this so that when the left edge of the window is 0,
    // coord 3 will be placed in the middle of the window.
    pub fn coord_to_center_x(coord: Coord) -> f32 {
        40. + 120. * coord as f32
    }

    pub fn position_objects<T>(
        objects: &[ObjectRenderData],
        models: &mut impl ModelAccess<T>,
    ) -> Vec<(Vec2, ObjectRenderData)> {
        let mut positioned_objects = Vec::new();
        let mut positioner = Positioner::default();
        let mut groups_cache: Vec<Vec<ObjectRenderData>> =
            vec![Vec::new(); (objects.iter().map(|obj| obj.coord).max().unwrap_or(0) + 1) as usize];

        let mut objects = objects.to_owned();
        objects.sort_by(|data1, data2| {
            let z_offset1 = models.lookup_model(&data1.model_id).z_offset;
            let z_offset2 = models.lookup_model(&data2.model_id).z_offset;
            data2
                .weight
                .cmp(&data1.weight)
                .then(z_offset1.cmp(&z_offset2))
        });

        for data in objects {
            let object_group = &mut groups_cache[data.coord as usize];
            if object_group
                .first()
                .is_some_and(|cached_object| cached_object.model_id != data.model_id)
            {
                positioned_objects
                    .extend(positioner.position_object_group(mem::take(object_group), models));
            }

            object_group.push(data);
        }

        for object_group in groups_cache {
            positioned_objects.extend(positioner.position_object_group(object_group, models));
        }

        positioned_objects.sort_by(|((_, z1), data1), ((_, z2), data2)| {
            data2
                .weight
                .cmp(&data1.weight)
                .then(z1.cmp(z2))
                .then(data1.coord.cmp(&data2.coord))
        });
        positioned_objects
            .into_iter()
            .map(|((pos, _), data)| (pos, data))
            .collect()
    }

    #[derive(Default)]
    pub struct Positioner {
        coord_counts: HashMap<Coord, (u16, i16)>,
    }

    impl Positioner {
        fn position_object_group<T>(
            &mut self,
            object_group: Vec<ObjectRenderData>,
            models: &mut impl ModelAccess<T>,
        ) -> Vec<((Vec2, i16), ObjectRenderData)> {
            if let Some((coord, model)) = object_group
                .first()
                .map(|object| (object.coord, models.lookup_model(&object.model_id)))
            {
                self.position_groups_from_offsets(
                    model.group_placement.position(object_group.len() as u16),
                    coord,
                    model,
                )
                .into_iter()
                .zip(object_group)
                .collect()
            } else {
                Vec::default()
            }
        }

        pub fn position_groups_from_offsets<T>(
            &mut self,
            offset_groups: Vec<Offsets>,
            coord: Coord,
            model: &Model<T>,
        ) -> Vec<(Vec2, i16)> {
            offset_groups
                .into_iter()
                .flat_map(|offsets| {
                    let (x, z) = self.position_object(coord, model);
                    offsets.into_iter().map(move |offset| {
                        (
                            (x + f32::from(offset.0), z + f32::from(offset.1)),
                            -z as i16,
                        )
                    })
                })
                .collect()
        }

        pub fn position_object<T>(&mut self, coord: Coord, model: &Model<T>) -> Vec2 {
            let (x_count, z_displacement) = self.coord_counts.entry(coord).or_insert((0, 0));

            let pos = position_from_coord(coord, *x_count, *z_displacement, model.z_offset);

            if model.has_x_displacement {
                *x_count += 1;
            }
            *z_displacement += model.z_displacement;
            pos
        }
    }

    fn position_from_coord(
        coord: Coord,
        x_displacement_count: u16,
        z_displacement: i16,
        z_offset: i16,
    ) -> Vec2 {
        (
            coord_to_center_x(coord) - f32::from(x_displacement_count * 15),
            f32::from(190 - z_displacement - z_offset),
        )
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
pub struct AftikProfile {
    pub name: String,
    pub color: AftikColorId,
    pub stats: Stats,
    #[serde(default, skip_serializing_if = "Traits::is_empty")]
    pub traits: Traits,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileOrRandom {
    #[default]
    Random,
    #[serde(untagged)]
    Profile(AftikProfile),
}

impl ProfileOrRandom {
    pub(crate) fn is_default(&self) -> bool {
        matches!(self, Self::Random)
    }

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
    let chosen_index = rng.random_range(0..character_profiles.len());
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
