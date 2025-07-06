mod ui {
    use aftiktuna::asset::color::AftikColorData;
    use aftiktuna::asset::location::creature::{
        self, AftikCorpseData, AttributeChoice, CreatureSpawnData, NpcSpawnData,
        ShopkeeperSpawnData,
    };
    use aftiktuna::asset::location::{
        AreaData, ContainerData, ContainerType, DoorAdjective, DoorSpawnData, DoorType, SymbolData,
        SymbolMap,
    };
    use aftiktuna::core::area::BackgroundId;
    use aftiktuna::core::display::{AftikColorId, ModelId};
    use aftiktuna_editor_three_d::name_from_symbol;
    use indexmap::IndexMap;
    use three_d::egui;

    const SYMBOL_LABEL_FONT: egui::FontId = egui::FontId::monospace(12.);

    pub fn editor_panels(
        editor_data: &mut super::EditorData,
        assets: &super::Assets,
        egui_context: &egui::Context,
    ) -> bool {
        let mut save = false;
        side_panel(egui_context, |ui| {
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

                if !editor_data.is_in_overview {
                    editor_data.symbol_edit_data = selection_ui(
                        ui,
                        &mut editor_data.location_data.areas,
                        &mut editor_data.area_index,
                        &assets.background_types,
                        &assets.base_symbols,
                    );
                }

                ui.separator();
                save = ui.button("Save").clicked();
            }
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

    fn selection_ui(
        ui: &mut egui::Ui,
        areas: &mut [AreaData],
        area_index: &mut usize,
        background_types: &[BackgroundId],
        base_symbols: &SymbolMap,
    ) -> Option<SymbolEditData> {
        egui::ComboBox::from_id_salt("area").show_index(ui, area_index, areas.len(), |index| {
            areas[index].name.clone()
        });
        ui.separator();

        let area = &mut areas[*area_index];
        let symbol_edit_data = area_editor_ui(ui, area, background_types, base_symbols);

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
        symbol_edit_data
    }

    fn area_editor_ui(
        ui: &mut egui::Ui,
        area: &mut AreaData,
        background_types: &[BackgroundId],
        base_symbols: &SymbolMap,
    ) -> Option<SymbolEditData> {
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
                ui.add(egui::Slider::new(offset, 0..=20));
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add Left").clicked() {
                area.objects.insert(0, String::default());
            }
            if ui.button("Add Right").clicked() {
                area.objects.push(String::default());
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Remove Left").clicked() {
                area.objects.remove(0);
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

            if ui.button("Add Inanimate").clicked() {
                symbol_edit_data = Some(SymbolEditData {
                    old_char: None,
                    new_char: String::new(),
                    symbol_data: SymbolData::Inanimate {
                        model: ModelId::unknown(),
                        direction: Default::default(),
                    },
                })
            }

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
            SymbolData::Item { item } => {
                aftiktuna_editor_three_d::item_type_editor(ui, item, "item");
            }
            SymbolData::Loot { table } => {
                aftiktuna_editor_three_d::loot_table_editor(ui, table);
            }
            SymbolData::Door(DoorSpawnData {
                pair_id,
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
                    .selected_text(container_type.noun().singular())
                    .show_ui(ui, |ui| {
                        for selectable_type in ContainerType::variants() {
                            ui.selectable_value(
                                container_type,
                                *selectable_type,
                                selectable_type.noun().singular(),
                            );
                        }
                    });
                aftiktuna_editor_three_d::direction_editor(ui, direction, "container_direction");
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
            .selected_text(creature.noun().singular())
            .show_ui(ui, |ui| {
                for selectable_type in creature::Type::variants() {
                    ui.selectable_value(
                        creature,
                        *selectable_type,
                        selectable_type.noun().singular(),
                    );
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

        ui.checkbox(wandering, "Wandering");

        aftiktuna_editor_three_d::option_direction_editor(ui, direction, "creature_direction");
    }
}

use aftiktuna::asset::color::{self, AftikColorData};
use aftiktuna::asset::location::{self, AreaData, LocationData, SymbolMap};
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::{background, placement};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::AftikColorId;
use aftiktuna::core::position::Coord;
use aftiktuna::location::generate::Symbols;
use aftiktuna_three_d::asset::LazilyLoadedModels;
use aftiktuna_three_d::{asset, render};
use indexmap::IndexMap;
use std::fs::{self, File};

const SIDE_PANEL_WIDTH: u32 = 250;
const BOTTOM_PANEL_HEIGHT: u32 = 30;

const SIZE: (u32, u32) = (
    aftiktuna_three_d::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    aftiktuna_three_d::WINDOW_HEIGHT as u32 + BOTTOM_PANEL_HEIGHT,
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
        area_index: 0,
        symbol_edit_data: None,
        is_in_overview: false,
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

        let area = &editor_data.location_data.areas[editor_data.area_index];
        if !editor_data.is_in_overview {
            camera.handle_inputs(&mut frame_input.events);
            camera.clamp(area.objects.len() as Coord);
        }

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: BOTTOM_PANEL_HEIGHT as i32,
            width: aftiktuna_three_d::WINDOW_WIDTH.into(),
            height: aftiktuna_three_d::WINDOW_HEIGHT.into(),
        };
        if editor_data.is_in_overview {
            render_overview(
                &editor_data.location_data,
                editor_data.area_index,
                render_viewport,
                &screen,
                &frame_input.context,
            );
        } else {
            render_game_view(
                area,
                &camera,
                render_viewport,
                &screen,
                &frame_input.context,
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
    area_index: usize,
    symbol_edit_data: Option<ui::SymbolEditData>,
    is_in_overview: bool,
}

struct Assets {
    background_types: Vec<BackgroundId>,
    background_map: asset::BackgroundMap,
    base_symbols: SymbolMap,
    models: LazilyLoadedModels,
    aftik_colors: IndexMap<AftikColorId, AftikColorData>,
}

const OVERVIEW_SCALE: f32 = 6.;

const AREA_COLOR: three_d::Vec4 = three_d::vec4(0.5, 0.5, 0.5, 0.5);
const SELECTED_AREA_COLOR: three_d::Vec4 = three_d::vec4(0.8, 0.8, 0.8, 0.8);

fn render_overview(
    location: &LocationData,
    selected_index: usize,
    render_viewport: three_d::Viewport,
    screen: &three_d::RenderTarget<'_>,
    context: &three_d::Context,
) {
    let center = three_d::vec2(
        render_viewport.width as f32 / 2.,
        render_viewport.height as f32 / 2.,
    );
    let objects = location
        .areas
        .iter()
        .enumerate()
        .map(|(index, area)| {
            let color = if index == selected_index {
                SELECTED_AREA_COLOR
            } else {
                AREA_COLOR
            };

            three_d::Gm::new(
                {
                    three_d::Rectangle::new(
                        context,
                        center
                            + three_d::vec2(
                                area.pos_in_overview.0 as f32,
                                area.pos_in_overview.1 as f32,
                            ) * OVERVIEW_SCALE,
                        three_d::degrees(0.),
                        area.objects.len() as f32 * OVERVIEW_SCALE,
                        OVERVIEW_SCALE,
                    )
                },
                render::color_material(color),
            )
        })
        .collect::<Vec<_>>();

    let render_camera = render::default_render_camera(render_viewport);
    render::draw_in_order(&objects, &render_camera, screen);
}

fn render_game_view(
    area: &AreaData,
    camera: &aftiktuna_three_d::Camera,
    render_viewport: three_d::Viewport,
    screen: &three_d::RenderTarget<'_>,
    context: &three_d::Context,
    assets: &mut Assets,
) {
    let backgorund_data = assets.background_map.get_or_default(&area.background);
    let background = render::render_objects_for_primary_background(
        backgorund_data,
        area.background_offset.unwrap_or(0),
        camera.camera_x,
        context,
    );
    let symbol_lookup = Symbols::new(&assets.base_symbols, &area.symbols);

    let mut objects = area
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
                    )
                })
        })
        .collect::<Vec<_>>();
    objects.sort_by(|data1, data2| data2.weight.cmp(&data1.weight));
    let objects = placement::position_objects(&objects, &mut assets.models);
    let objects = objects
        .into_iter()
        .flat_map(|(pos, object)| {
            render::get_render_objects_for_entity_with_color(
                assets.models.lookup_model(&object.model_id),
                pos.into(),
                object
                    .properties
                    .aftik_color
                    .as_ref()
                    .and_then(|color_id| assets.aftik_colors.get(color_id).copied())
                    .unwrap_or(color::DEFAULT_COLOR),
                &object.properties,
                context,
            )
        })
        .collect::<Vec<_>>();

    let render_camera = render::get_render_camera(camera, render_viewport);
    render::draw_in_order(&background, &render_camera, screen);
    render::draw_in_order(&objects, &render_camera, screen);
}
