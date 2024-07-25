use std::fs::{self, File};
use std::path::Path;
use std::process::exit;

use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::macroquad_interface;
use aftiktuna::macroquad_interface::camera::{HorizontalDraggableCamera, Positioner};
use aftiktuna::macroquad_interface::egui::EguiWrapper;
use aftiktuna::macroquad_interface::texture::model::{ColorSource, Model, RawModel};
use aftiktuna::macroquad_interface::texture::{
    background, model, AftikColorData, CachedTextures, RGBColor, TextureLoader,
};
use aftiktuna::view::area::RenderProperties;
use macroquad::window::Conf;
use macroquad::{color, window};

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna Model Editor".to_string(),
        window_width: 1000,
        window_height: 600,
        window_resizable: false,
        icon: Some(macroquad_interface::logo()),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut textures = CachedTextures::default();

    let objects_directory = fs::canonicalize("./assets/texture/object").unwrap();
    let path = rfd::FileDialog::new()
        .set_title("Pick a Model file")
        .add_filter("JSON", &["json"])
        .set_directory(objects_directory)
        .pick_file();
    let Some(path) = path else {
        return;
    };

    let mut selected_model = model::load_raw_model_from_path(&path).unwrap();
    assert!(
        !selected_model.layers.is_empty(),
        "Layers must not be empty"
    );
    let mut selected_layer = 0;

    let aftik_model = model::load_model(&ModelId::aftik()).unwrap();
    let background = background::load_background_for_testing();
    let mut area_size = 7;
    let mut camera = HorizontalDraggableCamera::centered_on_position(0, area_size);
    camera.set_default_size_viewport(0, 0);

    let mut egui = EguiWrapper::init();

    loop {
        window::clear_background(color::BLACK);
        let mut is_mouse_over_panel = false;

        egui.ui(|ctx| {
            is_mouse_over_panel |= side_panel(
                ctx,
                &mut selected_model,
                &mut selected_layer,
                &path,
                &mut textures,
            );
        });

        camera.handle_drag(area_size, !is_mouse_over_panel);

        let model = selected_model.load(&mut textures).unwrap();
        macroquad::camera::set_camera(&camera);
        background.draw(0, &camera);
        area_size = draw_examples(&selected_model, &model, &aftik_model);
        macroquad::camera::set_default_camera();

        egui.draw();
        window::next_frame().await;
    }
}

const DEFAULT_AFTIK_COLOR: AftikColorData = AftikColorData {
    primary_color: RGBColor::new(148, 216, 0),
    secondary_color: RGBColor::new(255, 238, 153),
};

fn draw_examples(raw_model: &RawModel, model: &Model, aftik_model: &Model) -> Coord {
    let mut positioner = Positioner::new();
    let mut next_coord = 0;
    let mut get_and_move_coord = || {
        let coord = next_coord;
        next_coord += 2;
        coord
    };

    bidirectional(|direction| {
        model.draw(
            positioner.position_object(get_and_move_coord(), false),
            false,
            &RenderProperties {
                direction,
                ..Default::default()
            },
            &DEFAULT_AFTIK_COLOR,
        );
    });

    if model.is_displacing() {
        let coord = get_and_move_coord();
        model.draw(
            positioner.position_object(coord, true),
            false,
            &RenderProperties {
                ..Default::default()
            },
            &DEFAULT_AFTIK_COLOR,
        );
        aftik_model.draw(
            positioner.position_object(coord, true),
            false,
            &RenderProperties {
                ..Default::default()
            },
            &DEFAULT_AFTIK_COLOR,
        );

        let coord = get_and_move_coord();
        aftik_model.draw(
            positioner.position_object(coord, true),
            false,
            &RenderProperties {
                ..Default::default()
            },
            &DEFAULT_AFTIK_COLOR,
        );
        model.draw(
            positioner.position_object(coord, true),
            false,
            &RenderProperties {
                ..Default::default()
            },
            &DEFAULT_AFTIK_COLOR,
        );

        let coord = get_and_move_coord();
        for _ in 0..3 {
            model.draw(
                positioner.position_object(coord, true),
                false,
                &RenderProperties {
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
        }
    } else {
        bidirectional(|direction| {
            let coord = get_and_move_coord();
            model.draw(
                positioner.position_object(coord, false),
                false,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
            aftik_model.draw(
                positioner.position_object(coord, true),
                false,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
        })
    }

    if raw_model
        .layers
        .iter()
        .any(|layer| layer.conditions.if_cut.is_some())
    {
        bidirectional(|direction| {
            model.draw(
                positioner.position_object(get_and_move_coord(), false),
                false,
                &RenderProperties {
                    direction,
                    is_cut: true,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
        });
    }

    if raw_model
        .layers
        .iter()
        .any(|layer| layer.conditions.if_hurt.is_some())
    {
        bidirectional(|direction| {
            model.draw(
                positioner.position_object(get_and_move_coord(), false),
                false,
                &RenderProperties {
                    direction,
                    is_badly_hurt: true,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
        });
    }

    if raw_model
        .layers
        .iter()
        .any(|layer| layer.conditions.if_alive.is_some())
    {
        bidirectional(|direction| {
            model.draw(
                positioner.position_object(get_and_move_coord(), false),
                false,
                &RenderProperties {
                    direction,
                    is_alive: false,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
        });
    }

    if raw_model.wield_offset != (0, 0) {
        bidirectional(|direction| {
            let pos = positioner.position_object(get_and_move_coord(), false);
            aftik_model.draw(
                pos,
                false,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
            model.draw(
                pos,
                true,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                &DEFAULT_AFTIK_COLOR,
            );
        });
    }

    next_coord - 1
}

fn bidirectional(mut closure: impl FnMut(Direction)) {
    closure(Direction::Right);
    closure(Direction::Left);
}

fn side_panel(
    ctx: &egui::Context,
    model: &mut RawModel,
    selected_layer: &mut usize,
    path: impl AsRef<Path>,
    textures: &mut CachedTextures,
) -> bool {
    let response = egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(200.)
        .show(ctx, |ui| {
            ui.label("Wield offset:");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut model.wield_offset.0));
                ui.add(egui::DragValue::new(&mut model.wield_offset.1));
            });
            if ui.button("Clear Offset").clicked() {
                model.wield_offset = (0, 0);
            }

            ui.checkbox(&mut model.mounted, "Mounted / Background");

            ui.separator();

            ui.label("Layers:");

            for (layer_index, layer) in model.layers.iter().enumerate() {
                ui.add_enabled_ui(layer_index != *selected_layer, |ui| {
                    if ui.button(&layer.texture).clicked() {
                        *selected_layer = layer_index;
                    }
                });
            }

            let layer = &mut model.layers[*selected_layer];

            ui.separator();

            egui::ComboBox::from_label("Coloration")
                .selected_text(format!("{:?}", layer.color))
                .show_ui(ui, |ui| {
                    for color in [
                        ColorSource::Uncolored,
                        ColorSource::Primary,
                        ColorSource::Secondary,
                    ] {
                        ui.selectable_value(&mut layer.color, color, format!("{:?}", color));
                    }
                });

            ui.collapsing("Conditions", |ui| {
                add_option_condition_combo_box("If Cut", &mut layer.conditions.if_cut, ui);
                add_option_condition_combo_box("If Alive", &mut layer.conditions.if_alive, ui);
                add_option_condition_combo_box("If Hurt", &mut layer.conditions.if_hurt, ui);
            });

            ui.separator();

            if let Some((width, height)) = &mut layer.positioning.size {
                ui.label("Size:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(width));
                    ui.add(egui::DragValue::new(height));
                });
                if ui.button("Use Texture Size").clicked() {
                    layer.positioning.size = None;
                }
            } else if ui.button("Use Custom Size").clicked() {
                let texture = textures.load_texture(layer.texture_path()).unwrap();
                layer.positioning.size = Some((texture.width() as i16, texture.height() as i16));
            }

            ui.label("Y-offset:");
            ui.add(egui::DragValue::new(&mut layer.positioning.y_offset));

            ui.checkbox(&mut layer.positioning.fixed, "Fixed Direction");

            ui.separator();

            if ui.button("Save").clicked() {
                save_model(model, path);
                exit(0);
            }
        });
    response.response.contains_pointer()
}

fn option_condition_text(condition: Option<bool>) -> &'static str {
    match condition {
        None => "Irregardless",
        Some(true) => "True",
        Some(false) => "False",
    }
}

fn add_option_condition_combo_box(
    label: &str,
    current_value: &mut Option<bool>,
    ui: &mut egui::Ui,
) {
    egui::ComboBox::from_label(label)
        .selected_text(option_condition_text(*current_value))
        .show_ui(ui, |ui| {
            for value in [None, Some(true), Some(false)] {
                ui.selectable_value(current_value, value, option_condition_text(value));
            }
        });
}

fn save_model(model: &RawModel, path: impl AsRef<Path>) {
    let file = File::create(path).unwrap();
    serde_json_pretty::to_writer(file, model).unwrap();
}
