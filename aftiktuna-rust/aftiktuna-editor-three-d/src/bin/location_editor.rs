use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::{ProfileOrRandom, background, color, placement};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId, OrderWeight};
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::status::Health;
use aftiktuna::location::generate::creature::CharacterInteraction;
use aftiktuna::location::generate::{self, AreaData, LocationData, SymbolData, SymbolMap, Symbols};
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use aftiktuna_three_d::asset::LazilyLoadedModels;
use aftiktuna_three_d::{asset, render};
use std::collections::HashMap;
use std::fs::{self, File};
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

    let mut location_data =
        serde_json::from_reader::<_, LocationData>(File::open(path).unwrap()).unwrap();
    let mut area_index = 0;

    let window = three_d::Window::new(three_d::WindowSettings {
        title: "Aftiktuna: Location Editor".to_string(),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let background_types = background::load_index_map_backgrounds()
        .unwrap()
        .into_keys()
        .collect::<Vec<_>>();
    let mut assets = Assets {
        background_map: asset::BackgroundMap::load(window.gl()).unwrap(),
        base_symbols: generate::load_base_symbols().unwrap(),
        models: LazilyLoadedModels::new(window.gl()).unwrap(),
        aftik_colors: color::load_aftik_color_data().unwrap(),
    };
    let mut camera = aftiktuna_three_d::Camera::default();
    let mut gui = three_d::GUI::new(&window.gl());

    window.render_loop(move |mut frame_input| {
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                side_panel(egui_context, |ui| {
                    egui::ComboBox::from_id_salt("area").show_index(
                        ui,
                        &mut area_index,
                        location_data.areas.len(),
                        |index| location_data.areas[index].name.clone(),
                    );
                    ui.separator();
                    let area = &mut location_data.areas[area_index];
                    area_editor_ui(ui, area, &background_types, &assets.base_symbols);
                    ui.separator();
                    ui.collapsing("Global Symbols", |ui| {
                        for (char, symbol_data) in &assets.base_symbols {
                            let color = if area.symbols.contains_key(char) {
                                egui::Color32::DARK_GRAY
                            } else {
                                egui::Color32::GRAY
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{char} : {}",
                                    name_from_symbol(symbol_data)
                                ))
                                .font(SYMBOL_LABEL_FONT)
                                .color(color),
                            );
                        }
                    });
                });

                let area = &mut location_data.areas[area_index];
                bottom_panel(egui_context, |ui| {
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
            },
        );

        let area = &location_data.areas[area_index];
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

        three_d::FrameOutput::default()
    });
}

struct Assets {
    background_map: asset::BackgroundMap,
    base_symbols: SymbolMap,
    models: LazilyLoadedModels,
    aftik_colors: HashMap<AftikColorId, color::AftikColorData>,
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

fn area_editor_ui(
    ui: &mut egui::Ui,
    area: &mut AreaData,
    background_types: &[BackgroundId],
    base_symbols: &SymbolMap,
) {
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
        }
    });
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
