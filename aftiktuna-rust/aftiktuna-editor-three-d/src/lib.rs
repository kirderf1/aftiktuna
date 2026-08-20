pub mod editors;

use aftiktuna::asset::color::{SpeciesColorData, SpeciesColorEntry};
use aftiktuna::asset::location::creature::CharacterInteraction;
use aftiktuna::asset::location::{DoorPairMap, DoorType, SymbolData};
use aftiktuna::asset::profile::ProfileOrRandom;
use aftiktuna::core::SpeciesId;
use aftiktuna::core::display::{ModelId, SpeciesColorId};
use aftiktuna::core::item::ItemTypeId;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::status::Health;
use aftiktuna::view::area::{ObjectProperties, ObjectRenderData};
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Default)]
pub struct SpeciesColors(HashMap<SpeciesId, IndexMap<SpeciesColorId, SpeciesColorEntry>>);

impl SpeciesColors {
    pub fn lookup(
        &mut self,
        species_id: &SpeciesId,
        color_id: &SpeciesColorId,
    ) -> Option<SpeciesColorData> {
        self.find_or_load(species_id)
            .get(color_id)
            .map(|entry| entry.color_data)
    }

    pub fn keys(&mut self, species_id: &SpeciesId) -> impl Iterator<Item = &SpeciesColorId> {
        self.find_or_load(species_id).keys()
    }

    fn find_or_load(
        &mut self,
        species_id: &SpeciesId,
    ) -> &IndexMap<SpeciesColorId, SpeciesColorEntry> {
        self.0.entry(species_id.clone()).or_insert_with(|| {
            aftiktuna::asset::color::SPECIES_COLOR_DIR
                .load_index_map(species_id)
                .ok()
                .unwrap_or_default()
        })
    }
}

pub fn name_from_symbol(symbol_data: &SymbolData) -> String {
    match symbol_data {
        SymbolData::LocationEntry => "Landing Spot".to_string(),
        SymbolData::FortunaChest => "Fortuna Chest".to_string(),
        SymbolData::ShipControls { .. } => "Ship Controls".to_string(),
        SymbolData::FoodDeposit => "Food Deposit".to_string(),
        SymbolData::ShipDialogueSpot => "Ship Dialogue Spot".to_string(),
        SymbolData::Item { item } => format!("Item ({item})"),
        SymbolData::Loot { table } => format!("Loot ({})", table.0),
        SymbolData::Door(door_spawn_data) => format!("Door ({})", door_spawn_data.pair_id),
        SymbolData::Inanimate { model, .. } => format!("Object ({})", model.0),
        SymbolData::Container(container_data) => {
            format!("Container ({:?})", container_data.container_type)
        }
        SymbolData::Creature(creature_spawn_data) => {
            format!("Creature ({})", creature_spawn_data.creature)
        }
        SymbolData::Character(npc_spawn_data) => {
            let interaction = match &npc_spawn_data.interaction {
                CharacterInteraction::Recruitable { .. } => "recruitable",
                CharacterInteraction::Passenger { .. } => "passenger",
                CharacterInteraction::Talk { .. } => "talkable",
                CharacterInteraction::GivesHuntReward(_) => "hunt quest",
                CharacterInteraction::Shopkeeper { .. } => "shopkeeper",
                CharacterInteraction::Hostile { .. } => "hostile",
            };
            format!("NCP ({interaction})")
        }
        SymbolData::CharacterCorpse(_) => "Character Corpse".to_string(),
        SymbolData::Furnish { .. } => "Furnish".to_string(),
    }
}

pub fn object_from_symbol(
    symbol_data: &SymbolData,
    coord: Coord,
    area_size: Coord,
    door_pair_map: &DoorPairMap,
    is_ship: bool,
) -> ObjectRenderData {
    match symbol_data {
        SymbolData::LocationEntry => ObjectRenderData {
            coord,
            model_id: if is_ship {
                DoorType::Doorway.into()
            } else {
                ModelId::ship()
            },
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties::default(),
        },
        SymbolData::FortunaChest => ObjectRenderData {
            coord,
            model_id: ModelId::fortuna_chest(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties::default(),
        },
        SymbolData::ShipControls { direction } => ObjectRenderData {
            coord,
            model_id: ModelId::ship_controls(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties {
                direction: *direction,
                ..Default::default()
            },
        },
        SymbolData::FoodDeposit | SymbolData::ShipDialogueSpot => ObjectRenderData {
            coord,
            model_id: ModelId::small_unknown(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties::default(),
        },
        SymbolData::Item { item } => ObjectRenderData {
            coord,
            model_id: item.model_id(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties::default(),
        },
        SymbolData::Loot { .. } => ObjectRenderData {
            coord,
            model_id: ModelId::small_unknown(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties::default(),
        },
        SymbolData::Door(door_spawn_data) => ObjectRenderData {
            coord,
            model_id: door_spawn_data
                .model
                .clone()
                .unwrap_or_else(|| door_spawn_data.door_type.into()),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties {
                is_cut: door_pair_map
                    .get(&door_spawn_data.pair_id)
                    .is_some_and(|pair_data| pair_data.is_cut),
                ..Default::default()
            },
        },
        SymbolData::Inanimate { model, direction } => ObjectRenderData {
            coord,
            model_id: model.clone(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties {
                direction: *direction,
                ..Default::default()
            },
        },
        SymbolData::Container(container_data) => ObjectRenderData {
            coord,
            model_id: container_data.container_type.model_id(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties {
                direction: container_data.direction,
                ..Default::default()
            },
        },
        SymbolData::Creature(creature_spawn_data) => {
            let health = Health::from_fraction(creature_spawn_data.health);
            ObjectRenderData {
                coord,
                model_id: creature_spawn_data
                    .custom_model
                    .clone()
                    .unwrap_or_else(|| creature_spawn_data.creature.model_id()),
                hash: 0,
                is_controlled: false,
                name_data: None,
                wielded_item: None,
                interactions: Vec::default(),
                properties: ObjectProperties {
                    direction: creature_spawn_data
                        .direction
                        .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                    is_alive: Health::from_fraction(creature_spawn_data.health).is_alive(),
                    is_badly_hurt: health.is_badly_hurt(),
                    ..Default::default()
                },
            }
        }
        SymbolData::Character(npc_spawn_data) => ObjectRenderData {
            coord,
            model_id: model_id_from_profile(&npc_spawn_data.profile),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: npc_spawn_data
                .wielded_item
                .as_ref()
                .map(ItemTypeId::model_id),
            interactions: Vec::default(),
            properties: ObjectProperties {
                direction: npc_spawn_data
                    .direction
                    .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                species_color: color_from_profile(&npc_spawn_data.profile),
                ..Default::default()
            },
        },
        SymbolData::CharacterCorpse(corpse_data) => ObjectRenderData {
            coord,
            model_id: corpse_data.species.model_id(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties {
                direction: corpse_data
                    .direction
                    .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                species_color: corpse_data
                    .color
                    .clone()
                    .map(|color_id| (corpse_data.species.clone(), color_id)),
                is_alive: false,
                is_badly_hurt: true,
                ..Default::default()
            },
        },
        SymbolData::Furnish { .. } => ObjectRenderData {
            coord,
            model_id: ModelId::unknown(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties::default(),
        },
    }
}

fn color_from_profile(profile: &ProfileOrRandom) -> Option<(SpeciesId, SpeciesColorId)> {
    match profile {
        ProfileOrRandom::Random { .. } => None,
        ProfileOrRandom::Profile(profile) => Some((profile.species.clone(), profile.color.clone())),
    }
}

fn model_id_from_profile(profile: &ProfileOrRandom) -> ModelId {
    match profile {
        ProfileOrRandom::Random { species, .. } => species,
        ProfileOrRandom::Profile(character_profile) => &character_profile.species,
    }
    .model_id()
}
