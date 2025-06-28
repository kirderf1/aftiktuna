use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::{ProfileOrRandom, background, color, placement};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId, OrderWeight};
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::core::status::Health;
use aftiktuna::location::generate::{self, AreaData, LocationData, SymbolData, Symbols};
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use aftiktuna_three_d::asset::LazilyLoadedModels;
use aftiktuna_three_d::{asset, render};
use std::fs::{self, File};
use three_d::egui;

const SIDE_PANEL_WIDTH: u32 = 200;
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
    let background_map = asset::BackgroundMap::load(window.gl()).unwrap();
    let base_symbols = generate::load_base_symbols().unwrap();
    let mut models = LazilyLoadedModels::new(window.gl()).unwrap();
    let mut aftik_colors = color::load_aftik_color_data().unwrap();
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
                    area_editor_ui(ui, &mut location_data.areas[area_index], &background_types);
                });

                let area = &mut location_data.areas[area_index];
                bottom_panel(egui_context, |ui| {
                    ui.horizontal(|ui| {
                        for symbols in &mut area.objects {
                            ui.add(egui::TextEdit::singleline(symbols).desired_width(40.));
                        }
                    });
                });
            },
        );

        let area = &location_data.areas[area_index];
        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(area.objects.len());

        let backgorund_data = background_map.get_or_default(&area.background);
        let background = render::render_objects_for_primary_background(
            backgorund_data,
            area.background_offset.unwrap_or(0),
            camera.camera_x,
            &frame_input.context,
        );
        let symbol_lookup = Symbols::new(&base_symbols, &area.symbols);

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
        let objects = placement::position_objects(&objects, &mut models);
        let objects = objects
            .into_iter()
            .flat_map(|(pos, object)| {
                render::get_render_objects_for_entity(
                    models.lookup_model(&object.model_id),
                    pos.into(),
                    &object.properties,
                    &mut aftik_colors,
                    &frame_input.context,
                )
            })
            .collect::<Vec<_>>();

        let render_viewport = three_d::Viewport {
            x: 0,
            y: BOTTOM_PANEL_HEIGHT as i32,
            width: aftiktuna_three_d::WINDOW_WIDTH.into(),
            height: aftiktuna_three_d::WINDOW_HEIGHT.into(),
        };
        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));
        let render_camera = render::get_render_camera(&camera, render_viewport);
        render::draw_in_order(&background, &render_camera, &screen);
        render::draw_in_order(&objects, &render_camera, &screen);

        screen.write(|| gui.render()).unwrap();

        three_d::FrameOutput::default()
    });
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

fn area_editor_ui(ui: &mut egui::Ui, area: &mut AreaData, background_types: &[BackgroundId]) {
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
