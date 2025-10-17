pub mod background;
pub mod location;
pub mod model;

pub mod color {
    use super::Error;
    use crate::core::display::AftikColorId;
    use crate::core::name::Adjective;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

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

    #[derive(Clone, Serialize, Deserialize)]
    pub struct AftikColorEntry {
        pub adjective: Adjective,
        #[serde(flatten)]
        pub color_data: AftikColorData,
    }

    pub const AFTIK_COLORS_PATH: &str = "assets/aftik_colors.json";

    pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorEntry>, Error> {
        super::load_from_json(AFTIK_COLORS_PATH)
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

pub(crate) mod dialogue {
    use crate::core::behavior::CrewLossMemory;
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
    use crate::core::item::ItemType;
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
        item: ItemType,
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

        pub(crate) fn pick_loot_item(&self, rng: &mut impl Rng) -> ItemType {
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
    use crate::core::position::{Coord, Direction};
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
            let weight1 = models.lookup_model(&data1.model_id).order_weight;
            let weight2 = models.lookup_model(&data2.model_id).order_weight;
            weight2
                .cmp(&weight1)
                .then(data1.is_controlled.cmp(&data2.is_controlled))
        });

        for data in objects {
            let object_group = &mut groups_cache[data.coord as usize];
            if models.lookup_model(&data.model_id).large_displacement {
                positioned_objects
                    .extend(positioner.position_object_group(mem::take(object_group), models));
                if let Some(object_group) = data
                    .coord
                    .checked_add_signed(data.properties.direction.opposite().into())
                    .and_then(|coord| groups_cache.get_mut(coord as usize))
                {
                    positioned_objects
                        .extend(positioner.position_object_group(mem::take(object_group), models));
                }
                positioned_objects.push((
                    positioner.position_object(
                        data.coord,
                        data.properties.direction,
                        models.lookup_model(&data.model_id),
                    ),
                    data,
                ));
            } else {
                if object_group
                    .first()
                    .is_some_and(|cached_object| cached_object.model_id != data.model_id)
                {
                    positioned_objects
                        .extend(positioner.position_object_group(mem::take(object_group), models));
                }

                object_group.push(data);
            }
        }

        for object_group in groups_cache {
            positioned_objects.extend(positioner.position_object_group(object_group, models));
        }

        positioned_objects.sort_by(|((_, z1), data1), ((_, z2), data2)| {
            let weight1 = models.lookup_model(&data1.model_id).order_weight;
            let weight2 = models.lookup_model(&data2.model_id).order_weight;
            z1.cmp(z2)
                .then(weight2.cmp(&weight1))
                .then(data1.is_controlled.cmp(&data2.is_controlled))
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
            if let Some((coord, direction, model)) = object_group.first().map(|object| {
                (
                    object.coord,
                    object.properties.direction,
                    models.lookup_model(&object.model_id),
                )
            }) {
                self.position_groups_from_offsets(
                    model.group_placement.position(object_group.len() as u16),
                    coord,
                    direction,
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
            direction: Direction,
            model: &Model<T>,
        ) -> Vec<(Vec2, i16)> {
            offset_groups
                .into_iter()
                .flat_map(|offsets| {
                    let ((x, y), z) = self.position_object(coord, direction, model);
                    offsets
                        .into_iter()
                        .map(move |offset| ((x + f32::from(offset.0), y + f32::from(offset.1)), z))
                })
                .collect()
        }

        pub fn position_object<T>(
            &mut self,
            coord: Coord,
            direction: Direction,
            model: &Model<T>,
        ) -> (Vec2, i16) {
            let (x_count, z_displacement) = if model.large_displacement
                && let Some(coord2) = coord.checked_add_signed(direction.opposite().into())
            {
                self.calculate_displacement(&[coord, coord2], model)
            } else {
                self.calculate_displacement(&[coord], model)
            };

            let pos = position_from_coord(coord, x_count, z_displacement, model.z_offset);
            (pos, -pos.1 as i16)
        }

        fn calculate_displacement<T>(&mut self, range: &[Coord], model: &Model<T>) -> (u16, i16) {
            let (x_count, z_displacement) = range
                .iter()
                .map(|coord| self.coord_counts.get(coord).copied().unwrap_or_default())
                .fold(
                    (0, 0),
                    |(x_count1, z_displacement1), (x_count2, z_displacement2)| {
                        (x_count1.max(x_count2), z_displacement1.max(z_displacement2))
                    },
                );

            let updated_x_count = x_count + if model.has_x_displacement { 1 } else { 0 };
            let updated_z_displacement = z_displacement + model.z_displacement;

            for &coord in range {
                self.coord_counts
                    .insert(coord, (updated_x_count, updated_z_displacement));
            }

            (x_count, z_displacement)
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
use crate::core::name::{Adjective, NounData, NounId};
use crate::core::status::{Stats, Traits};
use rand::Rng;
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
        }
    }
}

pub trait TextureLoader<T, E> {
    fn load_texture(&mut self, name: String) -> Result<T, E>;
}

pub(crate) fn load_json_asset<T: DeserializeOwned>(path: impl Display) -> Result<T, Error> {
    load_from_json(format!("assets/{path}"))
}

pub(crate) fn load_from_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, Error> {
    let path = path.as_ref();
    let file = File::open(path).map_err(|error| Error::IO(path.to_owned(), error))?;
    let object =
        serde_json::from_reader(file).map_err(|error| Error::Json(path.to_owned(), error))?;
    Ok(object)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AftikProfile {
    pub name: String,
    pub color: AftikColorId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<Stats>,
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

pub(crate) fn load_character_profiles() -> Result<Vec<AftikProfile>, Error> {
    load_json_asset("character_profiles.json")
}

#[derive(Debug, Deserialize)]
pub(crate) struct CrewData {
    pub points: i32,
    pub crew: Vec<ProfileOrRandom>,
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

pub(crate) struct GameAssets {
    pub noun_data_map: NounDataMap,
    pub color_adjective_map: HashMap<AftikColorId, Adjective>,
}

impl GameAssets {
    pub fn load() -> Result<Self, Error> {
        Ok(Self {
            noun_data_map: NounDataMap::load()?,
            color_adjective_map: color::load_aftik_color_data()?
                .into_iter()
                .map(|(id, entry)| (id, entry.adjective))
                .collect(),
        })
    }
}
