use crate::SpeciesColors;
use aftiktuna::asset::location::creature::{
    AttributeChoice, CharacterCorpseData, CharacterInteraction, CreatureSpawnData, NpcSpawnData,
};
use aftiktuna::asset::location::{
    ContainerData, ContainerType, DoorAdjective, DoorSpawnData, DoorType, ItemOrLoot, SymbolData,
    SymbolMap,
};
use aftiktuna::asset::model;
use aftiktuna::asset::species::{CharacterSpeciesData, FaunaData};
use aftiktuna::core::behavior::Wandering;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::item::ItemTypeId;
use aftiktuna::core::position::Direction;
use aftiktuna::core::{SpeciesId, Tag};
use indexmap::IndexMap;
use std::collections::HashSet;
use three_d::egui;

const SYMBOL_LABEL_FONT: egui::FontId = egui::FontId::monospace(12.);

/// Displays list of available global symbols.
pub fn global_symbols_display(
    ui: &mut egui::Ui,
    global_symbols: &SymbolMap,
    overriding_symbols: &SymbolMap,
) {
    ui.collapsing("Global Symbols", |ui| {
        for (char, symbol_data) in global_symbols {
            let color = if overriding_symbols.contains_key(char) {
                egui::Color32::DARK_GRAY
            } else {
                egui::Color32::GRAY
            };
            ui.label(
                egui::RichText::new(format!("{char} : {}", crate::name_from_symbol(symbol_data)))
                    .font(SYMBOL_LABEL_FONT)
                    .color(color),
            );
        }
    });
}

/// Displays list of available symbols and buttons to add, edit or delete these.
pub fn local_symbols_editor(
    ui: &mut egui::Ui,
    symbols: &mut SymbolMap,
    global_symbols: &SymbolMap,
    item_type_list: &[ItemTypeId],
) -> Option<SymbolEditData> {
    ui.collapsing("Local Symbols", |ui| {
        let mut symbol_edit_data = None;
        let mut char_to_delete = None;

        for (char, symbol_data) in &*symbols {
            let color = if global_symbols.contains_key(char) {
                egui::Color32::LIGHT_GRAY
            } else {
                egui::Color32::GRAY
            };
            ui.label(
                egui::RichText::new(format!("{char} : {}", crate::name_from_symbol(symbol_data)))
                    .font(SYMBOL_LABEL_FONT)
                    .color(color),
            );

            ui.horizontal(|ui| {
                if ui.button("Edit").clicked() {
                    symbol_edit_data = Some(SymbolEditData {
                        old_char: Some(*char),
                        new_char: char.to_string(),
                        symbol_data: symbol_data.clone(),
                    });
                }
                if ui.button("Delete").clicked() {
                    char_to_delete = Some(*char);
                }
            });
        }

        ui.separator();

        ui.horizontal_wrapped(|ui| {
            if ui.button("Add Inanimate").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Inanimate {
                        model: ModelId::new("environment/"),
                        direction: Default::default(),
                    },
                })
            }

            if ui.button("Add Door").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Door(DoorSpawnData {
                        pair_id: String::default(),
                        door_type: DoorType::Door,
                        model: None,
                        adjective: None,
                    }),
                })
            }

            if ui.button("Add Item").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Item {
                        item: item_type_list[0].clone(),
                    },
                })
            }

            if ui.button("Add Creature").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Creature(CreatureSpawnData {
                        creature: SpeciesId::from("goblin"),
                        name: None,
                        custom_model: None,
                        health: 1.,
                        stats: None,
                        attribute: AttributeChoice::Random,
                        aggressive: None,
                        wandering: None,
                        tag: None,
                        direction: None,
                    }),
                })
            }

            if ui.button("Add Character").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Character(Box::new(NpcSpawnData {
                        profile: aftiktuna::asset::profile::ProfileOrRandom::Random {
                            species: SpeciesId::from("aftik"),
                            stats_bonus: 0,
                        },
                        health: 1.,
                        morale: 0.,
                        tag: None,
                        background: None,
                        interaction: CharacterInteraction::Recruitable {
                            will_request: false,
                        },
                        background_dialogue: None,
                        wielded_item: None,
                        direction: None,
                    })),
                })
            }

            if ui.button("Add Character Corpse").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::CharacterCorpse(CharacterCorpseData {
                        species: SpeciesId::from("aftik"),
                        color: None,
                        direction: None,
                    }),
                })
            }

            if ui.button("Add Container").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Container(ContainerData {
                        container_type: ContainerType::Cabinet,
                        content: Vec::new(),
                        direction: Direction::Right,
                    }),
                })
            }
        });

        if let Some(char_to_delete) = char_to_delete {
            symbols.shift_remove(&char_to_delete);
            None
        } else {
            symbol_edit_data
        }
    })
    .body_returned
    .unwrap_or_default()
}

pub struct SymbolEditData {
    pub old_char: Option<char>,
    pub new_char: String,
    pub symbol_data: SymbolData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolStatus {
    Unique,
    Conflicting,
    Overriding,
}

pub enum SymbolEditAction {
    Done,
    Cancel,
}

/// Displays settings to edit data for one specific symbol.
pub fn symbol_editor_ui(
    ui: &mut egui::Ui,
    symbol_edit_data: &mut SymbolEditData,
    area_symbols: &SymbolMap,
    global_symbols: &SymbolMap,
    species_colors: &SpeciesColors,
    fauna_data: &IndexMap<SpeciesId, FaunaData>,
    species_data: &IndexMap<SpeciesId, CharacterSpeciesData>,
    item_type_list: &[ItemTypeId],
    area_tags: &HashSet<Tag>,
) -> Option<SymbolEditAction> {
    ui.label(crate::name_from_symbol(&symbol_edit_data.symbol_data));

    ui.add(egui::TextEdit::singleline(&mut symbol_edit_data.new_char).char_limit(1));

    let status = symbol_edit_data
        .new_char
        .chars()
        .next()
        .map(|new_char| {
            if Some(new_char) != symbol_edit_data.old_char && area_symbols.contains_key(&new_char) {
                SymbolStatus::Conflicting
            } else if global_symbols.contains_key(&new_char) {
                SymbolStatus::Overriding
            } else {
                SymbolStatus::Unique
            }
        })
        .unwrap_or(SymbolStatus::Unique);

    if status == SymbolStatus::Conflicting {
        ui.label(
            egui::RichText::new("Character conflicts with existing").color(egui::Color32::RED),
        );
    } else if status == SymbolStatus::Overriding {
        ui.label(egui::RichText::new("Character overrides global").color(egui::Color32::YELLOW));
    }

    ui.separator();

    match &mut symbol_edit_data.symbol_data {
        SymbolData::LocationEntry => {}
        SymbolData::FortunaChest => {}
        SymbolData::ShipControls { direction } => {
            super::direction_editor(ui, direction, "ship_controls_direction");
        }
        SymbolData::FoodDeposit | SymbolData::ShipDialogueSpot => {}
        SymbolData::Item { item } => {
            super::item_type_editor(ui, item, "item", item_type_list);
        }
        SymbolData::Loot { table } => {
            super::loot_table_id_editor(ui, table);
        }
        SymbolData::Door(DoorSpawnData {
            pair_id: _,
            door_type,
            model,
            adjective,
        }) => {
            egui::ComboBox::from_label("Door Type")
                .selected_text(format!("{door_type:?}"))
                .show_ui(ui, |ui| {
                    for selectable_type in DoorType::variants() {
                        ui.selectable_value(
                            door_type,
                            *selectable_type,
                            format!("{selectable_type:?}"),
                        );
                    }
                });

            super::custom_model_editor(ui, model, || ModelId::from(*door_type));

            fn adjective_name(adjective: Option<DoorAdjective>) -> &'static str {
                adjective.map(DoorAdjective::word).unwrap_or("none")
            }
            egui::ComboBox::from_label("Adjective")
                .selected_text(adjective_name(*adjective))
                .show_ui(ui, |ui| {
                    for selectable_type in [None]
                        .into_iter()
                        .chain(DoorAdjective::variants().iter().copied().map(Some))
                    {
                        ui.selectable_value(
                            adjective,
                            selectable_type,
                            adjective_name(selectable_type),
                        );
                    }
                });
        }
        SymbolData::Inanimate { model, direction } => {
            ui.text_edit_singleline(&mut model.0);
            if !model::MODEL_DIR.file_path(model).exists() {
                ui.label(egui::RichText::new("Missing File").color(egui::Color32::YELLOW));
            }
            super::direction_editor(ui, direction, "inanimate_direction");
        }
        SymbolData::Container(ContainerData {
            container_type,
            content,
            direction,
        }) => {
            egui::ComboBox::from_label("Container Type")
                .selected_text(format!("{container_type:?}"))
                .show_ui(ui, |ui| {
                    for selectable_type in ContainerType::variants() {
                        ui.selectable_value(
                            container_type,
                            *selectable_type,
                            format!("{selectable_type:?}"),
                        );
                    }
                });
            super::direction_editor(ui, direction, "container_direction");
            ui.separator();

            for (index, item_or_loot) in content.iter_mut().enumerate() {
                super::item_or_loot_editor(ui, item_or_loot, index, item_type_list);
            }
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    content.push(ItemOrLoot::Item {
                        item: item_type_list[0].clone(),
                    });
                }
                if ui.button("Remove").clicked() {
                    content.pop();
                }
            });
        }
        SymbolData::Creature(creature_spawn_data) => {
            creature_spawn_data_editor(ui, creature_spawn_data, fauna_data, area_tags);
        }
        SymbolData::Character(npc_spawn_data) => {
            npc_spawn_data_editor(
                ui,
                npc_spawn_data,
                species_colors,
                species_data,
                item_type_list,
            );
        }
        SymbolData::CharacterCorpse(CharacterCorpseData {
            species,
            color,
            direction,
        }) => {
            super::species_editor(ui, species, "corpse_species", species_data.keys());

            egui::ComboBox::new("corpse_color", "Color")
                .selected_text(
                    color
                        .as_ref()
                        .map::<&str, _>(|color| &color.0)
                        .unwrap_or("random"),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(color, None, "random");
                    for selectable in species_colors.find_or_load(species).keys() {
                        ui.selectable_value(color, Some(selectable.clone()), &selectable.0);
                    }
                });
            super::option_direction_editor(ui, direction, "aftik_corpse_direction");
        }
        SymbolData::Furnish { template } => {}
    }

    ui.separator();

    ui.horizontal(|ui| {
        let done = ui
            .add_enabled(
                !symbol_edit_data.new_char.is_empty() && status != SymbolStatus::Conflicting,
                egui::Button::new("Done"),
            )
            .clicked();
        let cancel = ui.add(egui::Button::new("Cancel")).clicked();

        if cancel {
            Some(SymbolEditAction::Cancel)
        } else if done {
            Some(SymbolEditAction::Done)
        } else {
            None
        }
    })
    .inner
}

fn creature_spawn_data_editor<'a>(
    ui: &mut egui::Ui,
    CreatureSpawnData {
        creature,
        name,
        custom_model,
        health,
        stats,
        attribute,
        aggressive,
        wandering,
        tag,
        direction,
    }: &mut CreatureSpawnData,
    fauna_data: &IndexMap<SpeciesId, FaunaData>,
    area_tags: &HashSet<Tag>,
) {
    super::species_editor(ui, creature, "fauna", fauna_data.keys());

    super::option_with_checkbox(ui, name, "Custom name", String::new, |ui, name| {
        ui.text_edit_singleline(name);
    });

    super::custom_model_editor(ui, custom_model, || creature.model_id());

    ui.label("Health:");
    ui.add(egui::Slider::new(health, 0.0..=1.0));

    super::option_with_checkbox(
        ui,
        stats,
        "Custom stats",
        || fauna_data.get(creature).unwrap().default_stats,
        super::stats_editor,
    );

    fn attribute_name(attribute: AttributeChoice) -> &'static str {
        match attribute {
            AttributeChoice::None => "none",
            AttributeChoice::Random => "random",
            AttributeChoice::Attribute(creature_attribute) => creature_attribute.as_adjective(),
        }
    }
    egui::ComboBox::from_label("Attribute")
        .selected_text(attribute_name(*attribute))
        .show_ui(ui, |ui| {
            for selectable_type in AttributeChoice::variants() {
                ui.selectable_value(attribute, selectable_type, attribute_name(selectable_type));
            }
        });

    fn agression_name(agression: Option<bool>) -> &'static str {
        match agression {
            None => "default",
            Some(false) => "false",
            Some(true) => "true",
        }
    }
    egui::ComboBox::from_label("Agressiveness")
        .selected_text(agression_name(*aggressive))
        .show_ui(ui, |ui| {
            for selectable_type in [None, Some(false), Some(true)] {
                ui.selectable_value(aggressive, selectable_type, agression_name(selectable_type));
            }
        });

    super::option_with_checkbox(
        ui,
        wandering,
        "Wandering",
        || Wandering { area_tag: None },
        |ui, Wandering { area_tag }| {
            super::option_with_checkbox(
                ui,
                area_tag,
                "Area Tag",
                || {
                    area_tags
                        .iter()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| Tag(String::new()))
                },
                |ui, tag| {
                    egui::ComboBox::from_id_salt("wandering area tag")
                        .selected_text(&tag.0)
                        .show_ui(ui, |ui| {
                            for selectable_tag in area_tags {
                                if ui
                                    .selectable_label(selectable_tag == tag, &selectable_tag.0)
                                    .clicked()
                                {
                                    *tag = selectable_tag.clone();
                                }
                            }
                        });
                },
            );
        },
    );

    super::option_direction_editor(ui, direction, "creature_direction");
}

fn npc_spawn_data_editor(
    ui: &mut egui::Ui,
    NpcSpawnData {
        profile,
        health,
        morale,
        tag,
        background,
        interaction,
        background_dialogue,
        wielded_item,
        direction,
    }: &mut NpcSpawnData,
    species_colors: &SpeciesColors,
    species_data: &IndexMap<SpeciesId, CharacterSpeciesData>,
    item_type_list: &[ItemTypeId],
) {
    super::profile_or_random_editor(ui, profile, species_colors, species_data);

    ui.separator();

    ui.label("Health:");
    ui.add(egui::Slider::new(health, 0.0..=1.0));

    ui.label("Morale:");
    ui.add(egui::Slider::new(morale, -10.0..=10.0));

    super::option_with_checkbox(
        ui,
        wielded_item,
        "Wielding item",
        ItemTypeId::crowbar,
        |ui, wielded_item| {
            super::item_type_editor(ui, wielded_item, "character_wielded", item_type_list)
        },
    );

    super::option_direction_editor(ui, direction, "character_direction");
}
