use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::process::exit;

use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::macroquad_interface::texture::background::BGTexture;
use aftiktuna::macroquad_interface::texture::model::{ColorSource, Model, RawModel};
use aftiktuna::macroquad_interface::texture::{model, AftikColorData, RGBColor, TextureLoader};
use aftiktuna::macroquad_interface::{self, camera, texture};
use aftiktuna::view::area::RenderProperties;
use egui_macroquad::egui;
use egui_macroquad::macroquad::camera::Camera2D;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::texture::Texture2D;
use egui_macroquad::macroquad::window::Conf;
use egui_macroquad::macroquad::{self, color, window};

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna Model Editor".to_string(),
        window_width: 800,
        window_height: 600,
        window_resizable: false,
        icon: Some(macroquad_interface::logo()),
        ..Default::default()
    }
}

const AREA_SIZE: Coord = 10;

#[macroquad::main(config)]
async fn main() {
    let mut textures = CachedTextures(HashMap::new());

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

    let background = BGTexture::Repeating(texture::load_texture("background/forest").unwrap());
    let mut camera = camera::position_centered_camera(0, AREA_SIZE);
    let mut last_drag_pos = None;

    loop {
        window::clear_background(color::LIGHTGRAY);
        let mut is_mouse_over_panel = false;

        egui_macroquad::ui(|ctx| {
            is_mouse_over_panel |= side_panel(
                ctx,
                &mut selected_model,
                &mut selected_layer,
                &path,
                &mut textures,
            );
        });

        camera::try_drag_camera(
            &mut last_drag_pos,
            &mut camera,
            AREA_SIZE,
            !is_mouse_over_panel,
        );

        let model = selected_model.load(&mut textures).unwrap();
        macroquad::camera::set_camera(&Camera2D::from_display_rect(camera));
        draw_examples(&model, &background, camera);
        macroquad::camera::set_default_camera();

        egui_macroquad::draw();
        window::next_frame().await;
    }
}

const DEFAULT_AFTIK_COLOR: AftikColorData = AftikColorData {
    primary_color: RGBColor::new(148, 216, 0),
    secondary_color: RGBColor::new(255, 238, 153),
};

fn draw_examples(model: &Model, background: &BGTexture, camera: Rect) {
    background.draw(0, camera);

    model.draw(
        Vec2::new(160., 450.),
        false,
        &RenderProperties {
            direction: Direction::Right,
            ..Default::default()
        },
        &DEFAULT_AFTIK_COLOR,
    );
    model.draw(
        Vec2::new(400., 450.),
        false,
        &RenderProperties {
            direction: Direction::Left,
            ..Default::default()
        },
        &DEFAULT_AFTIK_COLOR,
    );
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

            ui.separator();

            if let Some((width, height)) = &mut layer.positioning.size {
                ui.label("Width:");
                ui.add(egui::DragValue::new(width));
                ui.label("Height:");
                ui.add(egui::DragValue::new(height));
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
    response.response.hovered()
}

struct CachedTextures(HashMap<String, Texture2D>);

impl TextureLoader for CachedTextures {
    fn load_texture(&mut self, name: String) -> Result<Texture2D, std::io::Error> {
        if let Some(texture) = self.0.get(&name) {
            return Ok(*texture);
        }

        let texture = texture::load_texture(&name)?;
        self.0.insert(name, texture);
        Ok(texture)
    }
}

fn save_model(model: &RawModel, path: impl AsRef<Path>) {
    let file = File::create(path).unwrap();
    serde_json_pretty::to_writer(file, model).unwrap();
}
