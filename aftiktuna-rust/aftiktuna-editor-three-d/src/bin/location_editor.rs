mod ui {
    use aftiktuna::asset::color::AftikColorData;
    use aftiktuna::asset::location::creature::{
        self, AftikCorpseData, AttributeChoice, CreatureSpawnData, NpcSpawnData,
        ShopkeeperSpawnData, Wandering,
    };
    use aftiktuna::asset::location::{
        AreaData, ContainerData, ContainerType, DoorAdjective, DoorSpawnData, DoorType, ItemOrLoot,
        SymbolData, SymbolMap,
    };
    use aftiktuna::core::BlockType;
    use aftiktuna::core::area::BackgroundId;
    use aftiktuna::core::display::{AftikColorId, ModelId};
    use aftiktuna::core::item::ItemType;
    use aftiktuna::core::position::Direction;
    use aftiktuna_editor_three_d::name_from_symbol;
    use indexmap::IndexMap;
    use std::mem;
    use three_d::egui;

    const SYMBOL_LABEL_FONT: egui::FontId = egui::FontId::monospace(12.);

    pub fn editor_panels(
        editor_data: &mut super::EditorData,
        assets: &super::Assets,
        egui_context: &egui::Context,
    ) -> bool {
        editor_data.hovered_door_pair = None;
        let mut save = false;
        side_panel(egui_context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(symbol_edit_data) = &mut editor_data.symbol_edit_data {
                    let area = &mut editor_data.location_data.areas[editor_data.area_index];
                    let old_char = symbol_edit_data.old_char;
                    let action = symbol_editor_ui(
                        ui,
                        symbol_edit_data,
                        |new_char| {
                            if Some(new_char) != old_char && area.symbols.contains_key(&new_char) {
                                SymbolStatus::Conflicting
                            } else if assets.base_symbols.contains_key(&new_char) {
                                SymbolStatus::Overriding
                            } else {
                                SymbolStatus::Unique
                            }
                        },
                        &assets.aftik_colors,
                    );

                    match action {
                        Some(SymbolEditAction::Done) => {
                            let new_char = symbol_edit_data.new_char.chars().next().unwrap();
                            area.symbols
                                .insert(new_char, symbol_edit_data.symbol_data.clone());

                            if let Some(old_char) = symbol_edit_data.old_char
                                && old_char != new_char
                            {
                                area.symbols.swap_remove(&old_char);
                                for objects in &mut area.objects {
                                    *objects = objects.replace(old_char, &new_char.to_string());
                                }
                            }
                            editor_data.symbol_edit_data = None;
                        }
                        Some(SymbolEditAction::Cancel) => {
                            editor_data.symbol_edit_data = None;
                        }
                        None => {}
                    }
                } else {
                    if ui.button("Swap View").clicked() {
                        editor_data.is_in_overview = !editor_data.is_in_overview;
                    }

                    if editor_data.is_in_overview {
                        ui.separator();
                        overview_ui(editor_data, ui);
                    } else {
                        ui.separator();
                        area_view_ui(
                            ui,
                            editor_data,
                            &assets.background_types,
                            &assets.base_symbols,
                        );
                    }

                    ui.separator();
                    save = ui.button("Save").clicked();
                }
            });
        });

        let area = { &mut editor_data.location_data.areas[editor_data.area_index] };
        bottom_panel(egui_context, |ui| {
            if !editor_data.is_in_overview {
                ui.add_enabled_ui(editor_data.symbol_edit_data.is_none(), |ui| {
                    ui.horizontal(|ui| {
                        for symbols in &mut area.objects {
                            ui.add(
                                egui::TextEdit::singleline(symbols)
                                    .desired_width(30.)
                                    .font(egui::TextStyle::Monospace),
                            );
                        }
                    });
                });
            }
        });

        save
    }

    fn side_panel(egui_context: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) {
        egui::SidePanel::right("side")
            .frame(egui::Frame::side_top_panel(&egui_context.style()).inner_margin(8.))
            .resizable(false)
            .exact_width(crate::SIDE_PANEL_WIDTH as f32)
            .show(egui_context, panel_contents);
    }

    fn bottom_panel(egui_context: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) {
        egui::TopBottomPanel::bottom("bottom")
            .frame(egui::Frame::side_top_panel(&egui_context.style()).inner_margin(8.))
            .resizable(false)
            .exact_height(crate::BOTTOM_PANEL_HEIGHT as f32)
            .show(egui_context, panel_contents);
    }

    fn overview_ui(editor_data: &mut super::EditorData, ui: &mut egui::Ui) {
        for (door_pair, pair_data) in &mut editor_data.location_data.door_pairs {
            let hovered_label = ui.label(door_pair).interact(egui::Sense::hover()).hovered();

            fn block_type_name(block_type: Option<BlockType>) -> String {
                block_type
                    .map(|block_type| format!("{block_type:?}"))
                    .unwrap_or("None".to_owned())
            }
            let hovered_selection = egui::ComboBox::new(door_pair, "Block Type")
                .selected_text(block_type_name(pair_data.block_type))
                .show_ui(ui, |ui| {
                    for selectable_type in [None]
                        .into_iter()
                        .chain(BlockType::variants().iter().copied().map(Some))
                    {
                        ui.selectable_value(
                            &mut pair_data.block_type,
                            selectable_type,
                            block_type_name(selectable_type),
                        );
                    }
                })
                .response
                .hovered();
            ui.checkbox(&mut pair_data.is_cut, "Is Cut");

            let is_connecting = matches!(&editor_data.connecting_pair, Some((connecting_pair, _)) if connecting_pair == door_pair);
            if ui
                .add_enabled(!is_connecting, egui::Button::new("Connect"))
                .clicked()
            {
                editor_data.connecting_pair = Some((door_pair.clone(), Vec::new()));
            }

            if hovered_label || hovered_selection {
                editor_data.hovered_door_pair = Some(door_pair.clone());
            }
        }

        ui.separator();
        ui.label("New Door Pair:");
        let text_edit_response = ui.text_edit_singleline(&mut editor_data.new_door_pair_name);
        if text_edit_response.lost_focus()
            && ui.input(|input_state| input_state.key_pressed(egui::Key::Enter))
        {
            let new_door_pair_id = mem::take(&mut editor_data.new_door_pair_name);
            editor_data
                .location_data
                .door_pairs
                .entry(new_door_pair_id)
                .or_default();
        }
    }

    fn area_view_ui(
        ui: &mut egui::Ui,
        editor_data: &mut super::EditorData,
        background_types: &[BackgroundId],
        base_symbols: &SymbolMap,
    ) {
        egui::ComboBox::from_id_salt("area").show_index(
            ui,
            &mut editor_data.area_index,
            editor_data.location_data.areas.len(),
            |index| editor_data.location_data.areas[index].name.clone(),
        );

        if ui.button("New Area").clicked() {
            editor_data.location_data.areas.insert(
                editor_data.area_index + 1,
                AreaData {
                    name: String::new(),
                    pos_in_overview: (0, 0),
                    tag: None,
                    background: BackgroundId::blank(),
                    background_offset: None,
                    extra_background_layers: Vec::default(),
                    darkness: 0.,
                    objects: vec![String::default()],
                    symbols: SymbolMap::new(),
                },
            );
            editor_data.area_index += 1;
        }
        ui.separator();

        let area = &mut editor_data.location_data.areas[editor_data.area_index];
        editor_data.symbol_edit_data = area_editor_ui(
            ui,
            area,
            &mut editor_data.selected_extra_background_layer,
            background_types,
            base_symbols,
        );

        ui.separator();
        ui.collapsing("Global Symbols", |ui| {
            for (char, symbol_data) in base_symbols {
                let color = if area.symbols.contains_key(char) {
                    egui::Color32::DARK_GRAY
                } else {
                    egui::Color32::GRAY
                };
                ui.label(
                    egui::RichText::new(format!("{char} : {}", name_from_symbol(symbol_data)))
                        .font(SYMBOL_LABEL_FONT)
                        .color(color),
                );
            }
        });
    }

    fn area_editor_ui(
        ui: &mut egui::Ui,
        area: &mut AreaData,
        selected_extra_background_layer: &mut usize,
        background_types: &[BackgroundId],
        base_symbols: &SymbolMap,
    ) -> Option<SymbolEditData> {
        ui.label("Display Name:");
        ui.text_edit_singleline(&mut area.name);

        ui.label("Background:");
        egui::ComboBox::from_id_salt("background")
            .selected_text(&area.background.0)
            .show_ui(ui, |ui| {
                for background_id in background_types {
                    if ui
                        .selectable_label(background_id == &area.background, &background_id.0)
                        .clicked()
                    {
                        area.background = background_id.clone();
                    }
                }
            });

        ui.label("Background offset:");
        ui.horizontal(|ui| {
            let mut has_offset = area.background_offset.is_some();
            ui.add(egui::Checkbox::without_text(&mut has_offset));
            if has_offset && area.background_offset.is_none() {
                area.background_offset = Some(0);
            }
            if !has_offset && area.background_offset.is_some() {
                area.background_offset = None;
            }
            if let Some(offset) = &mut area.background_offset {
                ui.add(egui::Slider::new(offset, -10..=10));
            }
        });

        ui.add(egui::Slider::new(&mut area.darkness, 0.0..=1.0));

        aftiktuna_editor_three_d::background_layer_list_editor(
            ui,
            selected_extra_background_layer,
            &mut area.extra_background_layers,
        );
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Add Left").clicked() {
                area.objects.insert(0, String::default());
                if let Some(offset) = &mut area.background_offset {
                    *offset += 1;
                }
                for background_layer in &mut area.extra_background_layers {
                    background_layer.offset.x += 120;
                }
            }
            if ui.button("Add Right").clicked() {
                area.objects.push(String::default());
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Remove Left").clicked() {
                area.objects.remove(0);
                if let Some(offset) = &mut area.background_offset {
                    *offset -= 1;
                }
                for background_layer in &mut area.extra_background_layers {
                    background_layer.offset.x -= 120;
                }
            }
            if ui.button("Remove Right").clicked() {
                area.objects.pop();
            }
        });

        ui.collapsing("Local Symbols", |ui| {
            let mut symbol_edit_data = None;
            let mut char_to_delete = None;

            for (char, symbol_data) in &area.symbols {
                let color = if base_symbols.contains_key(char) {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::GRAY
                };
                ui.label(
                    egui::RichText::new(format!("{char} : {}", name_from_symbol(symbol_data)))
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
                            display_type: DoorType::Door,
                            adjective: None,
                        }),
                    })
                }

                if ui.button("Add Item").clicked() {
                    symbol_edit_data = Some(SymbolEditData {
                        old_char: None,
                        new_char: String::new(),
                        symbol_data: SymbolData::Item {
                            item: ItemType::MeteorChunk,
                        },
                    })
                }

                if ui.button("Add Creature").clicked() {
                    symbol_edit_data = Some(SymbolEditData {
                        old_char: None,
                        new_char: String::new(),
                        symbol_data: SymbolData::Creature(CreatureSpawnData {
                            creature: creature::Type::Goblin,
                            health: 1.,
                            attribute: AttributeChoice::Random,
                            aggressive: None,
                            wandering: None,
                            tag: None,
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
                area.symbols.shift_remove(&char_to_delete);
                None
            } else {
                symbol_edit_data
            }
        })
        .body_returned
        .unwrap_or_default()
    }

    pub struct SymbolEditData {
        old_char: Option<char>,
        new_char: String,
        symbol_data: SymbolData,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SymbolStatus {
        Unique,
        Conflicting,
        Overriding,
    }

    enum SymbolEditAction {
        Done,
        Cancel,
    }

    fn symbol_editor_ui(
        ui: &mut egui::Ui,
        symbol_edit_data: &mut SymbolEditData,
        symbol_lookup: impl FnOnce(char) -> SymbolStatus,
        aftik_colors: &IndexMap<AftikColorId, AftikColorData>,
    ) -> Option<SymbolEditAction> {
        ui.label(name_from_symbol(&symbol_edit_data.symbol_data));

        ui.add(egui::TextEdit::singleline(&mut symbol_edit_data.new_char).char_limit(1));

        let status = symbol_edit_data
            .new_char
            .chars()
            .next()
            .map(symbol_lookup)
            .unwrap_or(SymbolStatus::Unique);

        if status == SymbolStatus::Conflicting {
            ui.label(
                egui::RichText::new("Character conflicts with existing").color(egui::Color32::RED),
            );
        } else if status == SymbolStatus::Overriding {
            ui.label(
                egui::RichText::new("Character overrides global").color(egui::Color32::YELLOW),
            );
        }

        ui.separator();

        match &mut symbol_edit_data.symbol_data {
            SymbolData::LocationEntry => {}
            SymbolData::FortunaChest => {}
            SymbolData::ShipControls { direction } => {
                aftiktuna_editor_three_d::direction_editor(
                    ui,
                    direction,
                    "ship_controls_direction",
                );
            }
            SymbolData::FoodDeposit => {}
            SymbolData::Item { item } => {
                aftiktuna_editor_three_d::item_type_editor(ui, item, "item");
            }
            SymbolData::Loot { table } => {
                aftiktuna_editor_three_d::loot_table_id_editor(ui, table);
            }
            SymbolData::Door(DoorSpawnData {
                pair_id: _,
                display_type,
                adjective,
            }) => {
                egui::ComboBox::from_label("Door Type")
                    .selected_text(format!("{display_type:?}"))
                    .show_ui(ui, |ui| {
                        for selectable_type in DoorType::variants() {
                            ui.selectable_value(
                                display_type,
                                *selectable_type,
                                format!("{selectable_type:?}"),
                            );
                        }
                    });
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
                if !model.file_path().as_ref().exists() {
                    ui.label(egui::RichText::new("Missing File").color(egui::Color32::YELLOW));
                }
                aftiktuna_editor_three_d::direction_editor(ui, direction, "inanimate_direction");
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
                aftiktuna_editor_three_d::direction_editor(ui, direction, "container_direction");
                ui.separator();

                for (index, item_or_loot) in content.iter_mut().enumerate() {
                    aftiktuna_editor_three_d::item_or_loot_editor(ui, item_or_loot, index);
                }
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        content.push(ItemOrLoot::Item {
                            item: ItemType::AncientCoin,
                        });
                    }
                    if ui.button("Remove").clicked() {
                        content.pop();
                    }
                });
            }
            SymbolData::Creature(creature_spawn_data) => {
                creature_spawn_data_editor(ui, creature_spawn_data);
            }
            SymbolData::Shopkeeper(ShopkeeperSpawnData {
                stock,
                color,
                direction,
            }) => {
                aftiktuna_editor_three_d::color_editor(
                    ui,
                    color,
                    "shopkeeper_color",
                    aftik_colors.keys(),
                );
                aftiktuna_editor_three_d::option_direction_editor(
                    ui,
                    direction,
                    "shopkeeper_direction",
                );
            }
            SymbolData::Character(NpcSpawnData {
                profile,
                interaction,
                wielded_item,
                direction,
            }) => {
                aftiktuna_editor_three_d::option_direction_editor(
                    ui,
                    direction,
                    "character_direction",
                );
            }
            SymbolData::AftikCorpse(AftikCorpseData { color, direction }) => {
                egui::ComboBox::new("corpse_color", "Color")
                    .selected_text(
                        color
                            .as_ref()
                            .map::<&str, _>(|color| &color.0)
                            .unwrap_or("random"),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(color, None, "random");
                        for selectable in aftik_colors.keys() {
                            ui.selectable_value(color, Some(selectable.clone()), &selectable.0);
                        }
                    });
                aftiktuna_editor_three_d::option_direction_editor(
                    ui,
                    direction,
                    "aftik_corpse_direction",
                );
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

    fn creature_spawn_data_editor(
        ui: &mut egui::Ui,
        CreatureSpawnData {
            creature,
            health,
            attribute,
            aggressive,
            wandering,
            tag,
            direction,
        }: &mut CreatureSpawnData,
    ) {
        egui::ComboBox::from_label("Creature Type")
            .selected_text(format!("{creature:?}"))
            .show_ui(ui, |ui| {
                for selectable_type in creature::Type::variants() {
                    ui.selectable_value(creature, *selectable_type, format!("{selectable_type:?}"));
                }
            });
        ui.label("Health:");
        ui.add(egui::Slider::new(health, 0.0..=1.0));

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
                    ui.selectable_value(
                        attribute,
                        selectable_type,
                        attribute_name(selectable_type),
                    );
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
                    ui.selectable_value(
                        aggressive,
                        selectable_type,
                        agression_name(selectable_type),
                    );
                }
            });

        let mut is_wandering = wandering.is_some();
        ui.checkbox(&mut is_wandering, "Wandering");
        if is_wandering && wandering.is_none() {
            *wandering = Some(Wandering { area_tag: None });
        }
        if !is_wandering && wandering.is_some() {
            *wandering = None;
        }

        aftiktuna_editor_three_d::option_direction_editor(ui, direction, "creature_direction");
    }
}

use aftiktuna::asset::color::{self, AftikColorData};
use aftiktuna::asset::location::{
    self, AreaData, LocationData, SymbolData, SymbolLookup, SymbolMap,
};
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::{background, placement};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::AftikColorId;
use aftiktuna::core::position::Coord;
use aftiktuna_three_d::asset::{self, LazilyLoadedModels};
use aftiktuna_three_d::dimensions;
use aftiktuna_three_d::render::{self, RenderProperties};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs::{self, File};

const SIDE_PANEL_WIDTH: u32 = 250;
const BOTTOM_PANEL_HEIGHT: u32 = 30;

const SIZE: (u32, u32) = (
    dimensions::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    dimensions::WINDOW_HEIGHT as u32 + BOTTOM_PANEL_HEIGHT,
);

fn main() {
    let locations_directory = fs::canonicalize("./assets/location").unwrap();
    let path = rfd::FileDialog::new()
        .set_title("Pick a location file")
        .add_filter("JSON", &["json"])
        .set_directory(locations_directory)
        .pick_file();
    let Some(path) = path else {
        return;
    };

    let mut editor_data = EditorData {
        location_data: serde_json::from_reader::<_, LocationData>(File::open(&path).unwrap())
            .unwrap(),
        is_ship: path.ends_with("assets/location/crew_ship.json"),
        area_index: 0,
        selected_extra_background_layer: 0,
        symbol_edit_data: None,
        is_in_overview: false,
        dragged_area: None,
        hovered_door_pair: None,
        connecting_pair: None,
        new_door_pair_name: String::new(),
        mouse_pos: three_d::vec2(0., 0.),
    };

    let window = three_d::Window::new(three_d::WindowSettings {
        title: "Aftiktuna: Location Editor".to_string(),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let mut assets = Assets {
        background_types: background::load_index_map_backgrounds()
            .unwrap()
            .into_keys()
            .collect::<Vec<_>>(),
        background_map: asset::BackgroundMap::load(window.gl()).unwrap(),
        base_symbols: location::load_base_symbols().unwrap(),
        models: LazilyLoadedModels::new(window.gl()).unwrap(),
        aftik_colors: serde_json::from_reader::<_, _>(
            File::open(color::AFTIK_COLORS_PATH).unwrap(),
        )
        .unwrap(),
    };
    let mut camera = aftiktuna_three_d::Camera::default();
    let mut gui = three_d::GUI::new(&window.gl());

    window.render_loop(move |mut frame_input| {
        for event in &frame_input.events {
            if let three_d::Event::MouseMotion { position, .. } = event {
                editor_data.mouse_pos =
                    three_d::Vec2::from(*position) / frame_input.device_pixel_ratio;
            }
        }

        let mut save = false;

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                save = ui::editor_panels(&mut editor_data, &assets, egui_context);
            },
        );

        if editor_data.is_in_overview {
            handle_overview_input(
                &mut frame_input.events,
                frame_input.device_pixel_ratio,
                &mut editor_data,
            );
        } else {
            let area = &editor_data.location_data.areas[editor_data.area_index];
            camera.handle_inputs(&mut frame_input.events);
            camera.clamp(area.objects.len() as Coord);
        }

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: (frame_input.device_pixel_ratio * BOTTOM_PANEL_HEIGHT as f32) as i32,
            width: (frame_input.device_pixel_ratio * f32::from(dimensions::WINDOW_WIDTH)) as u32,
            height: (frame_input.device_pixel_ratio * f32::from(dimensions::WINDOW_HEIGHT)) as u32,
        };

        if editor_data.is_in_overview {
            render_overview(
                &editor_data,
                render_viewport,
                &screen,
                &frame_input.context,
                &assets.base_symbols,
            );
        } else {
            render_game_view(
                &editor_data,
                &camera,
                render_viewport,
                &frame_input,
                &mut assets,
            );
        }

        screen.write(|| gui.render()).unwrap();

        if save {
            let file = File::create(&path).unwrap();
            serde_json_pretty::to_writer(file, &editor_data.location_data).unwrap();

            three_d::FrameOutput {
                exit: true,
                ..Default::default()
            }
        } else {
            three_d::FrameOutput::default()
        }
    });
}

struct EditorData {
    location_data: LocationData,
    is_ship: bool,
    area_index: usize,
    selected_extra_background_layer: usize,
    symbol_edit_data: Option<ui::SymbolEditData>,
    is_in_overview: bool,
    dragged_area: Option<usize>,
    hovered_door_pair: Option<String>,
    connecting_pair: Option<(String, Vec<AreaSymbolId>)>,
    new_door_pair_name: String,
    mouse_pos: three_d::Vec2,
}

struct Assets {
    background_types: Vec<BackgroundId>,
    background_map: asset::BackgroundMap,
    base_symbols: SymbolMap,
    models: LazilyLoadedModels,
    aftik_colors: IndexMap<AftikColorId, AftikColorData>,
}

#[derive(Clone, Copy, Debug)]
struct AreaSymbolId {
    area_index: usize,
    symbol: char,
}

impl AreaSymbolId {
    fn get(self, location_data: &mut LocationData) -> &mut SymbolData {
        &mut location_data.areas[self.area_index].symbols[&self.symbol]
    }
}

fn handle_overview_input(
    events: &mut [three_d::Event],
    scale_factor: f32,
    editor_data: &mut EditorData,
) {
    for event in events {
        match event {
            three_d::Event::MousePress {
                button,
                position,
                handled,
                ..
            } => {
                if !*handled && *button == three_d::MouseButton::Left {
                    let (mouse_x, mouse_y) = mouse_to_overview_pos(*position, scale_factor);
                    let clicked_area = editor_data
                        .location_data
                        .areas
                        .iter()
                        .enumerate()
                        .find(|(_, area)| {
                            let offset = area_offset(area);
                            area.pos_in_overview.0 + offset <= mouse_x
                                && mouse_x
                                    < area.pos_in_overview.0 + offset + area.objects.len() as i32
                                && mouse_y == area.pos_in_overview.1
                        })
                        .map(|(index, _)| index);

                    if let Some((door_pair_id, current_selected)) = &mut editor_data.connecting_pair
                        && let Some(clicked_area) = clicked_area
                    {
                        let area = &editor_data.location_data.areas[clicked_area];
                        let offset = area_offset(area);
                        let clicked_coord = mouse_x - area.pos_in_overview.0 - offset;

                        if let Some(door_char) =
                            area.objects[clicked_coord as usize].chars().find(|char| {
                                matches!(area.symbols.get(char), Some(SymbolData::Door(_)))
                            })
                        {
                            current_selected.push(AreaSymbolId {
                                area_index: clicked_area,
                                symbol: door_char,
                            });
                            if let [door_id_1, door_id_2] = current_selected[..] {
                                connect_paths(
                                    &mut editor_data.location_data,
                                    door_pair_id.clone(),
                                    door_id_1,
                                    door_id_2,
                                );
                                editor_data.connecting_pair = None;
                            }
                        }
                    } else {
                        editor_data.dragged_area = clicked_area;
                    }
                    *handled = true;
                }
            }
            three_d::Event::MouseRelease {
                button, handled, ..
            } => {
                if editor_data.dragged_area.is_some() && *button == three_d::MouseButton::Left {
                    editor_data.dragged_area = None;
                    *handled = true;
                }
            }
            three_d::Event::MouseMotion {
                position, handled, ..
            } => {
                if !*handled && let Some(dragged_area) = editor_data.dragged_area {
                    let area = &mut editor_data.location_data.areas[dragged_area];
                    area.pos_in_overview = mouse_to_overview_pos(*position, scale_factor);
                    *handled = true;
                }
            }
            _ => {}
        }
    }
}

fn connect_paths(
    location_data: &mut LocationData,
    door_pair_id: String,
    door_id_1: AreaSymbolId,
    door_id_2: AreaSymbolId,
) {
    for area in &mut location_data.areas {
        for symbol_data in &mut area.symbols.values_mut() {
            if let SymbolData::Door(door_data) = symbol_data
                && door_data.pair_id == door_pair_id
            {
                door_data.pair_id = String::default();
            }
        }
    }

    if let SymbolData::Door(door_data) = door_id_1.get(location_data) {
        door_data.pair_id = door_pair_id.clone();
    }
    if let SymbolData::Door(door_data) = door_id_2.get(location_data) {
        door_data.pair_id = door_pair_id;
    }
}

const OVERVIEW_SCALE: f32 = 8.;

fn mouse_to_overview_pos(pos: three_d::PhysicalPoint, scale_factor: f32) -> (i32, i32) {
    (
        ((pos.x / scale_factor - dimensions::WINDOW_WIDTH_F / 2.) / OVERVIEW_SCALE).round() as i32,
        ((pos.y / scale_factor - BOTTOM_PANEL_HEIGHT as f32 - dimensions::WINDOW_HEIGHT_F / 2.)
            / OVERVIEW_SCALE)
            .round() as i32,
    )
}

const AREA_COLOR: three_d::Vec4 = three_d::vec4(0.2, 0.2, 0.2, 1.0);
const SELECTED_AREA_COLOR: three_d::Vec4 = three_d::vec4(0.5, 0.5, 0.5, 1.0);
const PATH_COLOR: three_d::Vec4 = three_d::vec4(0.4, 0.4, 0.7, 1.0);
const SPECIAL_COLOR: three_d::Vec4 = three_d::vec4(0.8, 0.0, 0.8, 1.0);
const ENTRY_COLOR: three_d::Vec4 = three_d::vec4(0.0, 0.0, 0.8, 1.0);
const NPC_COLOR: three_d::Vec4 = three_d::vec4(0.0, 0.8, 0.0, 1.0);
const RED_COLOR: three_d::Vec4 = three_d::vec4(0.8, 0.0, 0.0, 1.0);
const ITEM_COLOR: three_d::Vec4 = three_d::vec4(0.4, 0.7, 0.4, 1.0);

fn color_for_pos(objects: &str, selected: bool, symbol_lookup: &SymbolLookup) -> three_d::Vec4 {
    if objects
        .chars()
        .any(|char| matches!(symbol_lookup.lookup(char), Some(SymbolData::Door(_))))
    {
        PATH_COLOR
    } else if objects
        .chars()
        .any(|char| matches!(symbol_lookup.lookup(char), Some(SymbolData::FortunaChest)))
    {
        SPECIAL_COLOR
    } else if objects
        .chars()
        .any(|char| matches!(symbol_lookup.lookup(char), Some(SymbolData::LocationEntry)))
    {
        ENTRY_COLOR
    } else if objects.chars().any(|char| {
        matches!(
            symbol_lookup.lookup(char),
            Some(SymbolData::Character(_)) | Some(SymbolData::Shopkeeper(_))
        )
    }) {
        NPC_COLOR
    } else if objects
        .chars()
        .any(|char| matches!(symbol_lookup.lookup(char), Some(SymbolData::Creature(_))))
    {
        RED_COLOR
    } else if objects.chars().any(|char| {
        matches!(
            symbol_lookup.lookup(char),
            Some(SymbolData::Item { .. })
                | Some(SymbolData::Loot { .. })
                | Some(SymbolData::Container(_))
        )
    }) {
        ITEM_COLOR
    } else if selected {
        SELECTED_AREA_COLOR
    } else {
        AREA_COLOR
    }
}

fn render_overview(
    editor_data: &EditorData,
    render_viewport: three_d::Viewport,
    screen: &three_d::RenderTarget<'_>,
    context: &three_d::Context,
    base_symbols: &SymbolMap,
) {
    let location = &editor_data.location_data;
    let mut path_positions = HashMap::new();
    for door_pair_id in location.door_pairs.keys() {
        path_positions.insert(door_pair_id.clone(), Vec::new());
    }
    for area in &location.areas {
        let symbol_lookup = SymbolLookup::new(base_symbols, &area.symbols);
        let offset = area_offset(area);
        for (coord, objects) in area.objects.iter().enumerate() {
            for symbol in objects.chars() {
                if let Some(SymbolData::Door(door_spawn_data)) = symbol_lookup.lookup(symbol)
                    && let Some(positions) = path_positions.get_mut(&door_spawn_data.pair_id)
                {
                    positions.push((
                        area.pos_in_overview.0 + offset + coord as i32,
                        area.pos_in_overview.1,
                    ));
                }
            }
        }
    }

    let center = three_d::vec2(
        render_viewport.width as f32 / 2.,
        render_viewport.height as f32 / 2.,
    );
    let objects = location
        .areas
        .iter()
        .enumerate()
        .flat_map(|(index, area)| {
            let symbol_lookup = SymbolLookup::new(base_symbols, &area.symbols);
            let selected = index == editor_data.area_index;
            let offset = area_offset(area);

            area.objects
                .iter()
                .enumerate()
                .map(move |(coord, objects)| {
                    let color = color_for_pos(objects, selected, &symbol_lookup);
                    three_d::Gm::new(
                        three_d::Rectangle::new(
                            context,
                            center
                                + three_d::vec2(
                                    (area.pos_in_overview.0 + offset + coord as i32) as f32,
                                    area.pos_in_overview.1 as f32,
                                ) * OVERVIEW_SCALE,
                            three_d::degrees(0.),
                            OVERVIEW_SCALE,
                            OVERVIEW_SCALE,
                        ),
                        render::color_material(color),
                    )
                })
        })
        .collect::<Vec<_>>();
    let path_lines = path_positions
        .iter()
        .filter_map(|(pair_id, positions)| {
            if let [pos1, pos2] = positions[..] {
                let color = if location.door_pairs[pair_id].block_type.is_some() {
                    RED_COLOR
                } else {
                    PATH_COLOR
                };
                let thickness = if editor_data
                    .hovered_door_pair
                    .as_ref()
                    .is_some_and(|hovered_pair| hovered_pair == pair_id)
                {
                    3.
                } else {
                    1.
                };
                Some(three_d::Gm::new(
                    three_d::Line::new(
                        context,
                        center + three_d::vec2(pos1.0 as f32, pos1.1 as f32) * OVERVIEW_SCALE,
                        center + three_d::vec2(pos2.0 as f32, pos2.1 as f32) * OVERVIEW_SCALE,
                        thickness,
                    ),
                    render::color_material(color),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let render_camera = render::default_render_camera(render_viewport);
    render::draw_in_order(&path_lines, &render_camera, screen);
    render::draw_in_order(&objects, &render_camera, screen);
}

fn render_game_view(
    editor_data: &EditorData,
    camera: &aftiktuna_three_d::Camera,
    render_viewport: three_d::Viewport,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    let area = &editor_data.location_data.areas[editor_data.area_index];
    let extra_background_layers = assets
        .background_map
        .load_extra_layers(&area.extra_background_layers)
        .unwrap_or_default();
    let backgorund_data = assets.background_map.get_or_default(&area.background);
    let background = render::render_objects_for_primary_background(
        backgorund_data,
        area.background_offset.unwrap_or(0),
        camera.camera_x,
        &extra_background_layers,
        &frame_input.context,
    );
    let symbol_lookup = SymbolLookup::new(&assets.base_symbols, &area.symbols);

    let objects = area
        .objects
        .iter()
        .enumerate()
        .flat_map(|(coord, symbols)| {
            symbols
                .chars()
                .filter_map(|char| symbol_lookup.lookup(char))
                .map(move |symbol| {
                    aftiktuna_editor_three_d::object_from_symbol(
                        symbol,
                        coord as Coord,
                        area.objects.len() as Coord,
                        &editor_data.location_data.door_pairs,
                        editor_data.is_ship,
                    )
                })
        })
        .collect::<Vec<_>>();
    let objects = placement::position_objects(&objects, &mut assets.models);
    let objects = objects
        .into_iter()
        .flat_map(|(pos, object)| {
            let aftik_color = object
                .properties
                .aftik_color
                .as_ref()
                .and_then(|color_id| assets.aftik_colors.get(color_id).copied())
                .unwrap_or(color::DEFAULT_COLOR);
            render::get_render_objects_for_entity_with_color(
                assets.models.lookup_model(&object.model_id),
                pos.into(),
                RenderProperties {
                    object: &object.properties,
                    aftik_color,
                },
                frame_input.accumulated_time as f32,
                &frame_input.context,
            )
        })
        .collect::<Vec<_>>();

    let render_camera = render::get_render_camera(camera, render_viewport);
    let screen = frame_input.screen();
    render::draw_in_order(&background, &render_camera, &screen);
    render::draw_in_order(&objects, &render_camera, &screen);

    if area.darkness > 0. {
        render::render_darkness(
            editor_data.mouse_pos,
            200.,
            area.darkness,
            render_viewport,
            frame_input.device_pixel_ratio,
            &screen,
            &frame_input.context,
        );
    }
}

fn area_offset(area: &AreaData) -> i32 {
    -(area.objects.len() as i32) / 2
}
