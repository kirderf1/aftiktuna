use aftiktuna::asset::background;
use aftiktuna::asset::location::creature::CharacterInteraction;
use aftiktuna::asset::location::{DoorPairMap, DoorType, ItemOrLoot, SymbolData};
use aftiktuna::asset::loot::LootTableId;
use aftiktuna::asset::profile::ProfileOrRandom;
use aftiktuna::core::Species;
use aftiktuna::core::display::{ModelId, SpeciesColorId};
use aftiktuna::core::item::ItemTypeId;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::status::Health;
use aftiktuna::view::area::{ObjectProperties, ObjectRenderData};
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use three_d::egui;

pub fn direction_editor(ui: &mut egui::Ui, direction: &mut Direction, id: impl Hash) {
    egui::ComboBox::new(id, "Direction")
        .selected_text(format!("{direction:?}"))
        .show_ui(ui, |ui| {
            for selectable in [Direction::Left, Direction::Right] {
                ui.selectable_value(direction, selectable, format!("{selectable:?}"));
            }
        });
}

pub fn option_direction_editor(
    ui: &mut egui::Ui,
    direction: &mut Option<Direction>,
    id: impl Hash,
) {
    egui::ComboBox::new(id, "Direction")
        .selected_text(format!("{direction:?}"))
        .show_ui(ui, |ui| {
            for selectable in [None, Some(Direction::Left), Some(Direction::Right)] {
                ui.selectable_value(direction, selectable, format!("{selectable:?}"));
            }
        });
}

pub fn item_type_editor(
    ui: &mut egui::Ui,
    edited_type: &mut ItemTypeId,
    id: impl Hash,
    item_type_list: &[ItemTypeId],
) {
    egui::ComboBox::new(id, "Item Type")
        .selected_text(edited_type.to_string())
        .show_ui(ui, |ui| {
            for selectable_type in item_type_list {
                let mut response = ui
                    .selectable_label(edited_type == selectable_type, selectable_type.to_string());
                if response.clicked() && edited_type != selectable_type {
                    *edited_type = selectable_type.clone();
                    response.mark_changed();
                }
            }
        });
}

pub fn loot_table_id_editor(ui: &mut egui::Ui, loot_table_id: &mut LootTableId) {
    ui.text_edit_singleline(&mut loot_table_id.0);
    let path = ["assets", &loot_table_id.path()]
        .iter()
        .collect::<PathBuf>();
    if !path.exists() {
        ui.label(egui::RichText::new("Missing File").color(egui::Color32::YELLOW));
    }
}

pub fn item_or_loot_editor(
    ui: &mut egui::Ui,
    item_or_loot: &mut ItemOrLoot,
    id: impl Hash + Copy,
    item_type_list: &[ItemTypeId],
) {
    let selected_text = match item_or_loot {
        ItemOrLoot::Item { .. } => "Item",
        ItemOrLoot::Loot { .. } => "Loot",
    };
    egui::ComboBox::new(id, "Item or Loot")
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            let is_item = matches!(item_or_loot, ItemOrLoot::Item { .. });
            if ui.selectable_label(is_item, "Item").clicked() && !is_item {
                *item_or_loot = ItemOrLoot::Item {
                    item: item_type_list[0].clone(),
                };
            }
            let is_loot = matches!(item_or_loot, ItemOrLoot::Loot { .. });
            if ui.selectable_label(is_loot, "Loot").clicked() && !is_loot {
                *item_or_loot = ItemOrLoot::Loot {
                    table: LootTableId("resource".to_string()),
                };
            }
        });
    match item_or_loot {
        ItemOrLoot::Item { item } => {
            item_type_editor(ui, item, ("item_or_loot", id), item_type_list)
        }
        ItemOrLoot::Loot { table } => loot_table_id_editor(ui, table),
    }
}

pub fn color_editor<'a, I: Iterator<Item = &'a SpeciesColorId>>(
    ui: &mut egui::Ui,
    edited_color: &mut SpeciesColorId,
    id: impl Hash,
    species_colors: I,
) {
    egui::ComboBox::new(id, "Color")
        .selected_text(&edited_color.0)
        .show_ui(ui, |ui| {
            for selectable in species_colors {
                ui.selectable_value(edited_color, selectable.clone(), &selectable.0);
            }
        });
}

pub fn background_layer_list_editor(
    ui: &mut egui::Ui,
    selected_layer: &mut usize,
    layer_list: &mut Vec<background::ParallaxLayer<String>>,
) {
    egui::ComboBox::from_label("Background Layers")
        .selected_text(
            layer_list
                .get(*selected_layer)
                .map_or("", |layer| &layer.texture),
        )
        .show_ui(ui, |ui| {
            for (layer_index, layer) in layer_list.iter().enumerate() {
                ui.selectable_value(selected_layer, layer_index, &layer.texture);
            }
        });

    if ui.button("New Layer").clicked() {
        layer_list.push(background::ParallaxLayer {
            texture: "white_space".to_owned(),
            move_factor: 1.,
            is_looping: false,
            offset: background::Offset::default(),
        });
        *selected_layer = layer_list.len() - 1;
    }

    ui.separator();

    if let Some(layer) = layer_list.get_mut(*selected_layer) {
        background_layer_editor(ui, layer);
    }
}

fn background_layer_editor(ui: &mut egui::Ui, layer: &mut background::ParallaxLayer<String>) {
    if ui.button("Select Texture").clicked() {
        let textures_directory = fs::canonicalize("./assets/texture/background").unwrap();
        let path = rfd::FileDialog::new()
            .set_title("Pick a texture")
            .add_filter("PNG", &["png"])
            .set_directory(&textures_directory)
            .pick_file();

        if let Some(path) = path {
            let mut path = fs::canonicalize(path).unwrap();
            path.set_extension("");
            if let Ok(path) = path
                .strip_prefix(&textures_directory)
                .inspect_err(|error| eprintln!("Got error preparing path: {error}"))
            {
                layer.texture = path.to_str().unwrap().to_owned();
            }
        } else {
            println!("No valid path")
        }
    }

    ui.text_edit_singleline(&mut layer.texture);
    ui.label("Move Factor:");
    ui.add(egui::DragValue::new(&mut layer.move_factor).speed(0.01));
    ui.checkbox(&mut layer.is_looping, "Is Looping");
    ui.label("Offset:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut layer.offset.x));
        ui.add(egui::DragValue::new(&mut layer.offset.y));
    });
}

pub fn name_from_symbol(symbol_data: &SymbolData) -> String {
    match symbol_data {
        SymbolData::LocationEntry => "Landing Spot".to_string(),
        SymbolData::FortunaChest => "Fortuna Chest".to_string(),
        SymbolData::ShipControls { .. } => "Ship Controls".to_string(),
        SymbolData::FoodDeposit => "Food Deposit".to_string(),
        SymbolData::Item { item } => format!("Item ({item})"),
        SymbolData::Loot { table } => format!("Loot ({})", table.0),
        SymbolData::Door(door_spawn_data) => format!("Door ({})", door_spawn_data.pair_id),
        SymbolData::Inanimate { model, .. } => format!("Object ({})", model.0),
        SymbolData::Container(container_data) => {
            format!("Container ({:?})", container_data.container_type)
        }
        SymbolData::Creature(creature_spawn_data) => {
            format!("Creature ({:?})", creature_spawn_data.creature.species())
        }
        SymbolData::Character(npc_spawn_data) => {
            let interaction = match &npc_spawn_data.interaction {
                CharacterInteraction::Recruitable => "recruitable",
                CharacterInteraction::Talk { .. } => "talkable",
                CharacterInteraction::GivesHuntReward(_) => "hunt quest",
                CharacterInteraction::Shopkeeper { .. } => "shopkeeper",
                CharacterInteraction::Hostile { .. } => "hostile",
            };
            format!("NCP ({interaction})")
        }
        SymbolData::AftikCorpse(_) => "Aftik Corpse".to_string(),
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
        SymbolData::FoodDeposit => ObjectRenderData {
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
            model_id: door_spawn_data.display_type.into(),
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
                model_id: creature_spawn_data.creature.species().model_id(),
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
        SymbolData::AftikCorpse(aftik_corpse_data) => ObjectRenderData {
            coord,
            model_id: Species::Aftik.model_id(),
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: ObjectProperties {
                direction: aftik_corpse_data
                    .direction
                    .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                species_color: aftik_corpse_data
                    .color
                    .clone()
                    .map(|color_id| (Species::Aftik, color_id)),
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

fn color_from_profile(profile: &ProfileOrRandom) -> Option<(Species, SpeciesColorId)> {
    match profile {
        ProfileOrRandom::Random { .. } => None,
        ProfileOrRandom::Profile(aftik_profile) => {
            Some((Species::Aftik, aftik_profile.color.clone()))
        }
    }
}

fn model_id_from_profile(profile: &ProfileOrRandom) -> ModelId {
    match profile {
        ProfileOrRandom::Random { species } => *species,
        ProfileOrRandom::Profile(character_profile) => character_profile.species,
    }
    .species()
    .model_id()
}
