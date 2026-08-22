use aftiktuna::asset::location::{self, DoorPairMap, FurnishTemplate, SymbolLookup, SymbolMap};
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::species::{CharacterSpeciesData, FaunaData};
use aftiktuna::asset::{color, placement};
use aftiktuna::core::SpeciesId;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::item::ItemTypeId;
use aftiktuna::core::position::Coord;
use aftiktuna::view::area::ObjectProperties;
use aftiktuna_editor_three_d::SpeciesColors;
use aftiktuna_editor_three_d::editors::symbols::*;
use aftiktuna_three_d::asset::{self, LazilyLoadedModels};
use aftiktuna_three_d::dimensions;
use aftiktuna_three_d::render::{self, RenderProperties};
use indexmap::IndexMap;
use std::{collections::HashSet, fs::File};
use three_d::egui;

const SIDE_PANEL_WIDTH: u32 = 300;
const BOTTOM_PANEL_HEIGHT: u32 = 50;

const SIZE: (u32, u32) = (
    dimensions::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    dimensions::WINDOW_HEIGHT as u32 + BOTTOM_PANEL_HEIGHT,
);

pub fn run(file_path: std::path::PathBuf) {
    let template_data =
        serde_json::from_reader::<_, Vec<FurnishTemplate>>(File::open(&file_path).unwrap())
            .unwrap();

    let mut editor_data = EditorData {
        template_data,
        template_index: 0,
        symbol_edit_data: None,
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
        background_map: asset::BackgroundMap::load(window.gl()).unwrap(),
        global_symbols: location::BASE_SYMBOLS_FILE.load().unwrap(),
        models: LazilyLoadedModels::new(window.gl()).unwrap(),
        species: aftiktuna::asset::species::SPECIES_FILE
            .load_index_map()
            .unwrap(),
        fauna: aftiktuna::asset::species::FAUNA_FILE
            .load_index_map()
            .unwrap(),
        species_colors: SpeciesColors::default(),
        item_type_list: aftiktuna::asset::ITEM_TYPES_FILE
            .load_index_map()
            .unwrap()
            .into_keys()
            .collect(),
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
            |ui| {
                save = editor_panels(&mut editor_data, &mut assets, ui);
            },
        );

        let template = &editor_data.template_data[editor_data.template_index];
        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(template.objects.len() as Coord);

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: (frame_input.device_pixel_ratio * BOTTOM_PANEL_HEIGHT as f32) as i32,
            width: (frame_input.device_pixel_ratio * f32::from(dimensions::WINDOW_WIDTH)) as u32,
            height: (frame_input.device_pixel_ratio * f32::from(dimensions::WINDOW_HEIGHT)) as u32,
        };

        render_game_view(
            &editor_data,
            &camera,
            render_viewport,
            &frame_input,
            &mut assets,
        );

        screen.write(|| gui.render()).unwrap();

        if save {
            let file = File::create(&file_path).unwrap();
            serde_json_pretty::to_writer(file, &editor_data.template_data).unwrap();

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
    template_data: Vec<FurnishTemplate>,
    template_index: usize,
    symbol_edit_data: Option<SymbolEditData>,
    mouse_pos: three_d::Vec2,
}

struct Assets {
    background_map: asset::BackgroundMap,
    global_symbols: SymbolMap,
    models: LazilyLoadedModels,
    species: IndexMap<SpeciesId, CharacterSpeciesData>,
    fauna: IndexMap<SpeciesId, FaunaData>,
    species_colors: SpeciesColors,
    item_type_list: Vec<ItemTypeId>,
}

fn editor_panels(editor_data: &mut EditorData, assets: &mut Assets, ui: &mut egui::Ui) -> bool {
    let save = egui::Panel::right("side")
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(8.))
        .resizable(false)
        .exact_size(SIDE_PANEL_WIDTH as f32)
        .show_inside(ui, |ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| side_panel_content(ui, editor_data, assets))
                .inner
        })
        .inner;

    egui::Panel::bottom("bottom")
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(8.))
        .resizable(false)
        .exact_size(BOTTOM_PANEL_HEIGHT as f32)
        .show_inside(ui, |ui| {
            bottom_panel_content(ui, editor_data);
        });

    save
}

fn side_panel_content(
    ui: &mut egui::Ui,
    editor_data: &mut EditorData,
    assets: &mut Assets,
) -> bool {
    let mut save = false;
    if let Some(symbol_edit_data) = &mut editor_data.symbol_edit_data {
        let template = &mut editor_data.template_data[editor_data.template_index];
        let action = symbol_editor_ui(
            ui,
            symbol_edit_data,
            &template.symbols,
            &assets.global_symbols,
            &assets.species_colors,
            &assets.fauna,
            &assets.species,
            &assets.item_type_list,
            &HashSet::new(),
        );

        match action {
            Some(SymbolEditAction::Done) => {
                let new_char = symbol_edit_data.new_char.chars().next().unwrap();
                template
                    .symbols
                    .insert(new_char, symbol_edit_data.symbol_data.clone());

                if let Some(old_char) = symbol_edit_data.old_char
                    && old_char != new_char
                {
                    template.symbols.swap_remove(&old_char);
                    for objects in &mut template.objects {
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
        ui.horizontal(|ui| {
            if ui.button("Add Left").clicked() {
                for template in &mut editor_data.template_data {
                    template.objects.insert(0, String::new());
                }
            }
            if ui.button("Add Right").clicked() {
                for template in &mut editor_data.template_data {
                    template.objects.push(String::new());
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Remove Left").clicked() {
                for template in &mut editor_data.template_data {
                    template.objects.remove(0);
                }
            }
            if ui.button("Remove Right").clicked() {
                for template in &mut editor_data.template_data {
                    template.objects.pop();
                }
            }
        });

        ui.separator();

        egui::ComboBox::from_id_salt("template").show_index(
            ui,
            &mut editor_data.template_index,
            editor_data.template_data.len(),
            |index| format!("Alternative {}", index + 1),
        );

        if ui.button("New Alternative").clicked() {
            editor_data.template_data.push(FurnishTemplate {
                objects: vec![String::default()],
                symbols: SymbolMap::new(),
            });
            editor_data.template_index = editor_data.template_data.len() - 1;
        }

        ui.separator();

        let template = &mut editor_data.template_data[editor_data.template_index];
        editor_data.symbol_edit_data = local_symbols_editor(
            ui,
            &mut template.symbols,
            &assets.global_symbols,
            &assets.item_type_list,
        );

        ui.separator();
        global_symbols_display(ui, &assets.global_symbols, &template.symbols);

        ui.separator();
        save = ui.button("Save").clicked();
    }
    save
}

fn bottom_panel_content(ui: &mut egui::Ui, editor_data: &mut EditorData) {
    let template = &mut editor_data.template_data[editor_data.template_index];
    ui.add_enabled_ui(editor_data.symbol_edit_data.is_none(), |ui| {
        ui.horizontal(|ui| {
            for symbols in &mut template.objects {
                ui.add(
                    egui::TextEdit::singleline(symbols)
                        .desired_width(30.)
                        .font(egui::TextStyle::Monospace),
                );
            }
        });
    });
}

fn render_game_view(
    editor_data: &EditorData,
    camera: &aftiktuna_three_d::Camera,
    render_viewport: three_d::Viewport,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    let template = &editor_data.template_data[editor_data.template_index];
    let backgorund_data = assets.background_map.get_or_default(&BackgroundId::blank());
    let background = render::render_objects_for_primary_background(
        backgorund_data,
        0,
        camera.camera_x,
        &[],
        &frame_input.context,
    );
    let symbol_lookup = SymbolLookup::new(&assets.global_symbols, &template.symbols);

    let objects = template
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
                        template.objects.len() as Coord,
                        &DoorPairMap::new(),
                        false,
                    )
                })
        })
        .collect::<Vec<_>>();
    let objects = placement::position_objects(&objects, &mut assets.models);
    let objects = objects
        .into_iter()
        .flat_map(|(pos, object)| {
            let species_color = object
                .properties
                .species_color
                .as_ref()
                .and_then(|(species, color_id)| assets.species_colors.lookup(species, color_id))
                .unwrap_or(color::DEFAULT_COLOR);
            let mut render_objects = render::get_render_objects_for_entity_with_color(
                assets.models.lookup_model(&object.model_id),
                pos.into(),
                RenderProperties {
                    object: &object.properties,
                    species_color,
                },
                frame_input.accumulated_time as f32,
                &frame_input.context,
            );
            if let Some(item_model_id) = object.wielded_item {
                let item_model = assets.models.lookup_model(&item_model_id);
                let offset = aftiktuna_three_d::to_vec(
                    item_model.wield_offset,
                    object.properties.direction.into(),
                );
                render_objects.extend(render::get_render_objects_for_entity_with_color(
                    item_model,
                    three_d::Vec2::from(pos) + offset,
                    RenderProperties {
                        object: &ObjectProperties {
                            direction: object.properties.direction,
                            ..ObjectProperties::default()
                        },
                        species_color: color::DEFAULT_COLOR,
                    },
                    frame_input.accumulated_time as f32,
                    &frame_input.context,
                ));
            }
            render_objects
        })
        .collect::<Vec<_>>();

    let render_camera = render::get_render_camera(camera, render_viewport);
    let screen = frame_input.screen();
    render::draw_in_order(&background, &render_camera, &screen);
    render::draw_in_order(&objects, &render_camera, &screen);
}
