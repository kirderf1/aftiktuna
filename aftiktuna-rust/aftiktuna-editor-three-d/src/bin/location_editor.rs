use aftiktuna::asset::location::creature::{
    self, AftikCorpseData, AttributeChoice, CharacterInteraction, CreatureSpawnData, NpcSpawnData,
    ShopkeeperSpawnData,
};
use aftiktuna::asset::location::{
    self, AreaData, ContainerData, ContainerType, DoorAdjective, DoorSpawnData, DoorType,
    LocationData, SymbolData, SymbolMap,
};
use aftiktuna::asset::loot::LootTableId;
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::{ProfileOrRandom, background, color, placement};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId, OrderWeight};
use aftiktuna::core::item;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::status::Health;
use aftiktuna::location::generate::Symbols;
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use aftiktuna_three_d::asset::LazilyLoadedModels;
use aftiktuna_three_d::{asset, render};
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::Hash;
use std::path::PathBuf;
use three_d::egui;

const SIDE_PANEL_WIDTH: u32 = 250;
const BOTTOM_PANEL_HEIGHT: u32 = 30;

const SIZE: (u32, u32) = (
    aftiktuna_three_d::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    aftiktuna_three_d::WINDOW_HEIGHT as u32 + BOTTOM_PANEL_HEIGHT,
);

const SYMBOL_LABEL_FONT: egui::FontId = egui::FontId::monospace(12.);

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
        char_edit: None,
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
        aftik_colors: color::load_aftik_color_data().unwrap(),
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
                save = editor_panels(&mut editor_data, &assets, egui_context);
            },
        );

        let area = &editor_data.location_data.areas[editor_data.area_index];
        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(area.objects.len());

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: BOTTOM_PANEL_HEIGHT as i32,
            width: aftiktuna_three_d::WINDOW_WIDTH.into(),
            height: aftiktuna_three_d::WINDOW_HEIGHT.into(),
        };
        render_game_view(
            area,
            &camera,
            render_viewport,
            &screen,
            &frame_input.context,
            &mut assets,
        );

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
    char_edit: Option<(char, SymbolEditData)>,
}

struct Assets {
    background_types: Vec<BackgroundId>,
    background_map: asset::BackgroundMap,
    base_symbols: SymbolMap,
    models: LazilyLoadedModels,
    aftik_colors: HashMap<AftikColorId, color::AftikColorData>,
}

fn editor_panels(
    editor_data: &mut EditorData,
    assets: &Assets,
    egui_context: &egui::Context,
) -> bool {
    let mut save = false;
    let EditorData {
        location_data,
        area_index,
        char_edit,
    } = editor_data;
    side_panel(egui_context, |ui| {
        if let Some((char, symbol_edit_data)) = char_edit {
            let area = &mut location_data.areas[*area_index];
            let action = symbol_editor_ui(ui, symbol_edit_data, |new_char| {
                if new_char != *char && area.symbols.contains_key(&new_char) {
                    SymbolStatus::Conflicting
                } else if assets.base_symbols.contains_key(&new_char) {
                    SymbolStatus::Overriding
                } else {
                    SymbolStatus::Unique
                }
            });

            match action {
                Some(SymbolEditAction::Done) => {
                    let new_char = symbol_edit_data.new_char.chars().next().unwrap();
                    area.symbols
                        .insert(new_char, symbol_edit_data.symbol_data.clone());
                    if *char != new_char {
                        area.symbols.swap_remove(char);
                    }
                    for objects in &mut area.objects {
                        *objects = objects.replace(*char, &new_char.to_string());
                    }
                    *char_edit = None;
                }
                Some(SymbolEditAction::Cancel) => {
                    *char_edit = None;
                }
                None => {}
            }
        } else {
            let char_to_edit = selection_ui(
                ui,
                &mut location_data.areas,
                area_index,
                &assets.background_types,
                &assets.base_symbols,
            );

            ui.separator();
            save = ui.button("Save").clicked();

            if let Some(char_to_edit) = char_to_edit
                && let Some(symbol_data) = location_data.areas[*area_index]
                    .symbols
                    .get(&char_to_edit)
                    .cloned()
            {
                *char_edit = Some((
                    char_to_edit,
                    SymbolEditData {
                        new_char: char_to_edit.to_string(),
                        symbol_data,
                    },
                ));
            }
        }
    });

    let area = &mut location_data.areas[*area_index];
    bottom_panel(egui_context, |ui| {
        ui.add_enabled_ui(char_edit.is_none(), |ui| {
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
    });

    save
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
                .map(move |symbol| object_from_symbol(symbol, coord, area.objects.len()))
        })
        .collect::<Vec<_>>();
    objects.sort_by(|data1, data2| data2.weight.cmp(&data1.weight));
    let objects = placement::position_objects(&objects, &mut assets.models);
    let objects = objects
        .into_iter()
        .flat_map(|(pos, object)| {
            render::get_render_objects_for_entity(
                assets.models.lookup_model(&object.model_id),
                pos.into(),
                &object.properties,
                &mut assets.aftik_colors,
                context,
            )
        })
        .collect::<Vec<_>>();

    let render_camera = render::get_render_camera(camera, render_viewport);
    render::draw_in_order(&background, &render_camera, screen);
    render::draw_in_order(&objects, &render_camera, screen);
}

fn side_panel(egui_context: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&egui_context.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(SIDE_PANEL_WIDTH as f32)
        .show(egui_context, panel_contents);
}

fn bottom_panel(egui_context: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) {
    egui::TopBottomPanel::bottom("bottom")
        .frame(egui::Frame::side_top_panel(&egui_context.style()).inner_margin(8.))
        .resizable(false)
        .exact_height(BOTTOM_PANEL_HEIGHT as f32)
        .show(egui_context, panel_contents);
}

fn selection_ui(
    ui: &mut egui::Ui,
    areas: &mut [AreaData],
    area_index: &mut usize,
    background_types: &[BackgroundId],
    base_symbols: &SymbolMap,
) -> Option<char> {
    egui::ComboBox::from_id_salt("area").show_index(ui, area_index, areas.len(), |index| {
        areas[index].name.clone()
    });
    ui.separator();

    let area = &mut areas[*area_index];
    let char_to_edit = area_editor_ui(ui, area, background_types, base_symbols);

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
    char_to_edit
}

fn area_editor_ui(
    ui: &mut egui::Ui,
    area: &mut AreaData,
    background_types: &[BackgroundId],
    base_symbols: &SymbolMap,
) -> Option<char> {
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
        let mut char_to_edit = None;

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
            if ui.button("Edit").clicked() {
                char_to_edit = Some(*char);
            }
        }
        char_to_edit
    })
    .body_returned
    .unwrap_or_default()
}

struct SymbolEditData {
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
        ui.label(egui::RichText::new("Character overrides global").color(egui::Color32::YELLOW));
    }

    ui.separator();

    match &mut symbol_edit_data.symbol_data {
        SymbolData::LocationEntry => {}
        SymbolData::FortunaChest => {}
        SymbolData::Item { item } => {
            item_type_editor(ui, item, "item");
        }
        SymbolData::Loot { table } => {
            loot_table_editor(ui, table);
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
            direction_editor(ui, direction, "inanimate_direction");
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
            direction_editor(ui, direction, "container_direction");
        }
        SymbolData::Creature(creature_spawn_data) => {
            creature_spawn_data_editor(ui, creature_spawn_data);
        }
        SymbolData::Shopkeeper(ShopkeeperSpawnData {
            stock,
            color,
            direction,
        }) => {
            option_direction_editor(ui, direction, "shopkeeper_direction");
        }
        SymbolData::Character(NpcSpawnData {
            profile,
            interaction,
            direction,
        }) => {
            option_direction_editor(ui, direction, "character_direction");
        }
        SymbolData::AftikCorpse(AftikCorpseData { color, direction }) => {
            option_direction_editor(ui, direction, "aftik_corpse_direction");
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

    ui.checkbox(wandering, "Wandering");

    option_direction_editor(ui, direction, "creature_direction");
}

fn direction_editor(ui: &mut egui::Ui, direction: &mut Direction, id: impl Hash) {
    egui::ComboBox::new(id, "Direction")
        .selected_text(format!("{direction:?}"))
        .show_ui(ui, |ui| {
            for selectable in [Direction::Left, Direction::Right] {
                ui.selectable_value(direction, selectable, format!("{selectable:?}"));
            }
        });
}

fn option_direction_editor(ui: &mut egui::Ui, direction: &mut Option<Direction>, id: impl Hash) {
    egui::ComboBox::new(id, "Direction")
        .selected_text(format!("{direction:?}"))
        .show_ui(ui, |ui| {
            for selectable in [None, Some(Direction::Left), Some(Direction::Right)] {
                ui.selectable_value(direction, selectable, format!("{selectable:?}"));
            }
        });
}

fn item_type_editor(ui: &mut egui::Ui, edited_type: &mut item::Type, id: impl Hash) {
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

fn loot_table_editor(ui: &mut egui::Ui, loot_table_id: &mut LootTableId) {
    ui.text_edit_singleline(&mut loot_table_id.0);
    let path = ["assets", &loot_table_id.path()]
        .iter()
        .collect::<PathBuf>();
    if !path.exists() {
        ui.label(egui::RichText::new("Missing File").color(egui::Color32::YELLOW));
    }
}

fn name_from_symbol(symbol_data: &SymbolData) -> String {
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

fn object_from_symbol(
    symbol_data: &SymbolData,
    coord: Coord,
    area_size: Coord,
) -> ObjectRenderData {
    match symbol_data {
        SymbolData::LocationEntry => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: ModelId::ship(),
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::FortunaChest => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: ModelId::fortuna_chest(),
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Item { item } => ObjectRenderData {
            coord,
            weight: OrderWeight::Item,
            model_id: (*item).into(),
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Loot { .. } => ObjectRenderData {
            coord,
            weight: OrderWeight::Item,
            model_id: ModelId::small_unknown(),
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Door(door_spawn_data) => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: door_spawn_data.display_type.into(),
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties: RenderProperties::default(),
        },
        SymbolData::Inanimate { model, direction } => ObjectRenderData {
            coord,
            weight: OrderWeight::Background,
            model_id: model.clone(),
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
