use aftiktuna::asset::location::creature::CharacterInteraction;
use aftiktuna::asset::location::{DoorPairMap, SymbolData};
use aftiktuna::asset::loot::LootTableId;
use aftiktuna::asset::{ProfileOrRandom, background};
use aftiktuna::core::display::{AftikColorId, ModelId, OrderWeight};
use aftiktuna::core::item;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::status::Health;
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
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

pub fn item_type_editor(ui: &mut egui::Ui, edited_type: &mut item::Type, id: impl Hash) {
    egui::ComboBox::new(id, "Item Type")
        .selected_text(edited_type.noun_data().singular())
        .show_ui(ui, |ui| {
            for selectable_type in item::Type::variants() {
                ui.selectable_value(
                    edited_type,
                    *selectable_type,
                    selectable_type.noun_data().singular(),
                );
            }
        });
}

pub fn loot_table_editor(ui: &mut egui::Ui, loot_table_id: &mut LootTableId) {
    ui.text_edit_singleline(&mut loot_table_id.0);
    let path = ["assets", &loot_table_id.path()]
        .iter()
        .collect::<PathBuf>();
    if !path.exists() {
        ui.label(egui::RichText::new("Missing File").color(egui::Color32::YELLOW));
    }
}

pub fn color_editor<'a, I: Iterator<Item = &'a AftikColorId>>(
    ui: &mut egui::Ui,
    edited_color: &mut AftikColorId,
    id: impl Hash,
    aftik_colors: I,
) {
    egui::ComboBox::new(id, "Color")
        .selected_text(&edited_color.0)
        .show_ui(ui, |ui| {
            for selectable in aftik_colors {
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
        SymbolData::LocationEntry => "Landing Spot".to_owned(),
        SymbolData::FortunaChest => "Fortuna Chest".to_owned(),
        SymbolData::Item { item } => format!("Item ({})", item.noun_data().singular()),
        SymbolData::Loot { table } => format!("Loot ({})", table.0),
        SymbolData::Door(door_spawn_data) => format!("Door ({})", door_spawn_data.pair_id),
        SymbolData::Inanimate { model, .. } => format!("Object ({})", model.0),
        SymbolData::Container(container_data) => {
            format!(
                "Container ({})",
                container_data.container_type.noun().singular()
            )
        }
        SymbolData::Creature(creature_spawn_data) => {
            format!(
                "Creature ({})",
                creature_spawn_data.creature.noun().singular()
            )
        }
        SymbolData::Shopkeeper(_) => "Shopkeeper".to_owned(),
        SymbolData::Character(npc_spawn_data) => {
            let interaction = match &npc_spawn_data.interaction {
                CharacterInteraction::Recruitable => "recruitable",
                CharacterInteraction::GivesHuntReward(_) => "hunt quest",
            };
            format!("NCP ({interaction})")
        }
        SymbolData::AftikCorpse(_) => "Aftik Corpse".to_owned(),
    }
}

pub fn object_from_symbol(
    symbol_data: &SymbolData,
    coord: Coord,
    area_size: Coord,
    door_pair_map: &DoorPairMap,
) -> ObjectRenderData {
    match symbol_data {
        SymbolData::LocationEntry => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: ModelId::ship(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::FortunaChest => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: ModelId::fortuna_chest(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Item { item } => ObjectRenderData {
            coord,
            weight: OrderWeight::Item,
            model_id: (*item).into(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Loot { .. } => ObjectRenderData {
            coord,
            weight: OrderWeight::Item,
            model_id: ModelId::small_unknown(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Door(door_spawn_data) => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: door_spawn_data.display_type.into(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties {
                is_cut: door_pair_map
                    .get(&door_spawn_data.pair_id)
                    .is_some_and(|pair_data| pair_data.is_cut),
                ..Default::default()
            },
        },
        SymbolData::Inanimate { model, direction } => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: model.clone(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties {
                direction: *direction,
                ..Default::default()
            },
        },
        SymbolData::Container(container_data) => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: container_data.container_type.model_id(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties {
                direction: container_data.direction,
                ..Default::default()
            },
        },
        SymbolData::Creature(creature_spawn_data) => {
            let health = Health::from_fraction(creature_spawn_data.health);
            ObjectRenderData {
                coord,
                weight: OrderWeight::Creature,
                model_id: creature_spawn_data.creature.model_id(),
                hash: 0,
                name_data: None,
                wielded_item: None,
                interactions: Vec::default(),
                properties: RenderProperties {
                    direction: creature_spawn_data
                        .direction
                        .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                    is_alive: Health::from_fraction(creature_spawn_data.health).is_alive(),
                    is_badly_hurt: health.is_badly_hurt(),
                    ..Default::default()
                },
            }
        }
        SymbolData::Shopkeeper(shopkeeper_spawn_data) => ObjectRenderData {
            coord,
            weight: OrderWeight::Creature,
            model_id: ModelId::aftik(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties {
                direction: shopkeeper_spawn_data
                    .direction
                    .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                aftik_color: Some(shopkeeper_spawn_data.color.clone()),
                ..Default::default()
            },
        },
        SymbolData::Character(npc_spawn_data) => ObjectRenderData {
            coord,
            weight: OrderWeight::Creature,
            model_id: ModelId::aftik(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties {
                direction: npc_spawn_data
                    .direction
                    .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                aftik_color: color_from_profile(&npc_spawn_data.profile),
                ..Default::default()
            },
        },
        SymbolData::AftikCorpse(aftik_corpse_data) => ObjectRenderData {
            coord,
            weight: OrderWeight::Creature,
            model_id: ModelId::aftik(),
            hash: 0,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties {
                direction: aftik_corpse_data
                    .direction
                    .unwrap_or_else(|| Direction::between_coords(coord, (area_size - 1) / 2)),
                aftik_color: aftik_corpse_data.color.clone(),
                is_alive: false,
                is_badly_hurt: true,
                ..Default::default()
            },
        },
    }
}

fn color_from_profile(profile: &ProfileOrRandom) -> Option<AftikColorId> {
    match profile {
        ProfileOrRandom::Random => None,
        ProfileOrRandom::Profile(aftik_profile) => Some(aftik_profile.color.clone()),
    }
}
