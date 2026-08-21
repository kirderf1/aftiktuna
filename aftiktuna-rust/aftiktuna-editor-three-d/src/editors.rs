use crate::SpeciesColors;
use aftiktuna::asset::location::ItemOrLoot;
use aftiktuna::asset::loot::LootTableId;
use aftiktuna::asset::profile::{CharacterProfile, ProfileOrRandom, StatsOrRandom, TraitsOrRandom};
use aftiktuna::asset::species::CharacterSpeciesData;
use aftiktuna::asset::{background, loot, model};
use aftiktuna::core::SpeciesId;
use aftiktuna::core::display::{ModelId, SpeciesColorId};
use aftiktuna::core::item::ItemTypeId;
use aftiktuna::core::position::Direction;
use aftiktuna::core::status::Stats;
use indexmap::IndexMap;
use std::fs;
use std::hash::Hash;
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

pub fn species_editor<'a>(
    ui: &mut egui::Ui,
    edited_id: &mut SpeciesId,
    id: impl Hash,
    species_id_list: impl Iterator<Item = &'a SpeciesId>,
) {
    egui::ComboBox::new(id, "Species")
        .selected_text(edited_id.to_string())
        .show_ui(ui, |ui| {
            for selectable_id in species_id_list {
                let mut response =
                    ui.selectable_label(edited_id == selectable_id, selectable_id.to_string());
                if response.clicked() && edited_id != selectable_id {
                    *edited_id = selectable_id.clone();
                    response.mark_changed();
                }
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
    let path = loot::LOOT_TABLE_DIR.file_path(loot_table_id);
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

pub fn model_id_editor(ui: &mut egui::Ui, model_id: &mut ModelId) {
    if ui.button("Select Model").clicked() {
        let objects_directory = fs::canonicalize(model::MODEL_DIR.dir_path()).unwrap();
        let path = rfd::FileDialog::new()
            .set_title("Pick a model file")
            .add_filter("JSON", &["json"])
            .set_directory(&objects_directory)
            .pick_file();

        if let Some(path) = path {
            let mut path = fs::canonicalize(path).unwrap();
            path.set_extension("");
            if let Ok(path) = path
                .strip_prefix(&objects_directory)
                .inspect_err(|error| eprintln!("Got error preparing path: {error}"))
            {
                model_id.0 = path.to_str().unwrap().to_owned();
            }
        } else {
            println!("No valid path")
        }
    }

    ui.text_edit_singleline(&mut model_id.0);
}

pub fn option_with_checkbox<T>(
    ui: &mut egui::Ui,
    option: &mut Option<T>,
    label: &str,
    default: impl FnOnce() -> T,
    editor: impl FnOnce(&mut egui::Ui, &mut T),
) {
    let mut has_value: bool = option.is_some();
    if ui.checkbox(&mut has_value, label).changed() {
        *option = if has_value { Some(default()) } else { None };
    }
    if let Some(value) = option {
        editor(ui, value);
    }
}

pub fn stats_editor(
    ui: &mut egui::Ui,
    Stats {
        strength,
        endurance,
        agility,
        luck,
    }: &mut Stats,
) {
    ui.label("Strength:");
    ui.add(egui::Slider::new(strength, 1..=15));
    ui.label("Endurance:");
    ui.add(egui::Slider::new(endurance, 1..=15));
    ui.label("Agility:");
    ui.add(egui::Slider::new(agility, 1..=15));
    ui.label("Luck:");
    ui.add(egui::Slider::new(luck, 0..=15));
}

pub fn stats_or_random_editor(
    ui: &mut egui::Ui,
    stats: &mut StatsOrRandom,
    default_stats: impl FnOnce() -> Stats,
) {
    let mut is_random: bool = matches!(stats, StatsOrRandom::Random { .. });
    if ui.checkbox(&mut is_random, "Random Stats").changed() {
        *stats = match stats {
            StatsOrRandom::Random { stats_bonus } => {
                let mut stats = default_stats();
                let _ = stats.adjust_random_in_bounds(*stats_bonus, &mut rand::rng());
                StatsOrRandom::Stats(stats)
            }
            StatsOrRandom::Stats(stats) => StatsOrRandom::Random {
                stats_bonus: stats.sum() - default_stats().sum(),
            },
        }
    }

    match stats {
        StatsOrRandom::Random { stats_bonus } => {
            ui.label("Stats bonus:");
            ui.add(egui::Slider::new(stats_bonus, -15..=15));
        }
        StatsOrRandom::Stats(stats) => stats_editor(ui, stats),
    }
}

pub fn character_profile_editor(
    ui: &mut egui::Ui,
    CharacterProfile {
        species,
        name,
        color,
        stats,
        traits,
    }: &mut CharacterProfile,
    species_colors: &SpeciesColors,
    species_data: &IndexMap<SpeciesId, CharacterSpeciesData>,
) {
    species_editor(ui, species, "character_species", species_data.keys());

    ui.text_edit_singleline(name);

    egui::ComboBox::new("profile_color", "Color")
        .selected_text(&color.0)
        .show_ui(ui, |ui| {
            for selectable in species_colors.find_or_load(species).keys() {
                ui.selectable_value(color, selectable.clone(), &selectable.0);
            }
        });

    stats_or_random_editor(ui, stats, || {
        species_data.get(species).unwrap().default_stats
    });
}

pub fn profile_or_random_editor(
    ui: &mut egui::Ui,
    profile: &mut ProfileOrRandom,
    species_colors: &SpeciesColors,
    species_data: &IndexMap<SpeciesId, CharacterSpeciesData>,
) {
    let mut is_random: bool = matches!(profile, ProfileOrRandom::Random { .. });
    if ui.checkbox(&mut is_random, "Random Profile").changed() {
        *profile = match profile {
            ProfileOrRandom::Random {
                species,
                stats_bonus,
            } => ProfileOrRandom::Profile(CharacterProfile {
                species: species.clone(),
                name: String::new(),
                color: species_colors
                    .find_or_load(species)
                    .keys()
                    .next()
                    .unwrap()
                    .clone(),
                stats: StatsOrRandom::Random {
                    stats_bonus: *stats_bonus,
                },
                traits: TraitsOrRandom::Random,
            }),
            ProfileOrRandom::Profile(CharacterProfile { species, stats, .. }) => {
                let stats_bonus = match stats {
                    StatsOrRandom::Random { stats_bonus } => *stats_bonus,
                    StatsOrRandom::Stats(_) => 0,
                };
                ProfileOrRandom::Random {
                    species: species.clone(),
                    stats_bonus,
                }
            }
        };
    }

    match profile {
        ProfileOrRandom::Random {
            species,
            stats_bonus,
        } => {
            species_editor(ui, species, "character_species", species_data.keys());

            ui.label("Stats bonus:");
            ui.add(egui::Slider::new(stats_bonus, -15..=15));
        }
        ProfileOrRandom::Profile(character_profile) => {
            character_profile_editor(ui, character_profile, species_colors, species_data);
        }
    }
}

pub fn custom_model_editor(
    ui: &mut egui::Ui,
    custom_model: &mut Option<ModelId>,
    default: impl FnOnce() -> ModelId,
) {
    option_with_checkbox(ui, custom_model, "Custom model", default, model_id_editor);
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
