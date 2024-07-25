use std::fs::File;
use std::process::exit;

use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::position::Coord;
use aftiktuna::macroquad_interface;
use aftiktuna::macroquad_interface::camera::HorizontalDraggableCamera;
use aftiktuna::macroquad_interface::egui::EguiWrapper;
use aftiktuna::macroquad_interface::texture::background::{BGData, RawBGData};
use aftiktuna::macroquad_interface::texture::{background, CachedTextures};
use indexmap::IndexMap;
use macroquad::color;
use macroquad::window::{self, Conf};

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

    let mut backgrounds = background::load_index_map_backgrounds().unwrap();
    let mut selected_bg = 0;
    let mut selected_layer = 0;

    let mut area_size = 5;
    let mut offset = 0;
    let mut camera = HorizontalDraggableCamera::centered_on_position(0, area_size);
    camera.set_default_size_viewport(0, 0);

    let mut egui = EguiWrapper::init();

    loop {
        window::clear_background(color::BLACK);

        let mut is_mouse_over_panel = false;
        egui.ui(|ctx| {
            is_mouse_over_panel |= side_panel(ctx, |ui| {
                display_parameters_ui(&mut area_size, &mut offset, ui);

                ui.separator();

                background_editor_ui(&mut selected_bg, &mut selected_layer, &mut backgrounds, ui);
            });
        });

        camera.clamp(area_size);
        camera.handle_drag(area_size, !is_mouse_over_panel);

        macroquad::camera::set_camera(&camera);
        let (_, raw_background) = backgrounds.get_index(selected_bg).unwrap();
        draw_examples(
            &raw_background.load(&mut textures).unwrap(),
            offset,
            &camera,
        );
        macroquad::camera::set_default_camera();

        egui.draw();
        window::next_frame().await;
    }
}

fn draw_examples(background: &BGData, offset: Coord, camera: &HorizontalDraggableCamera) {
    background.texture.draw(offset, camera);
}

fn side_panel(ctx: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) -> bool {
    let response = egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(200.)
        .show(ctx, panel_contents);
    response.response.hovered()
}

fn display_parameters_ui(area_size: &mut Coord, offset: &mut Coord, ui: &mut egui::Ui) {
    ui.label("Area Size:");
    ui.add(egui::Slider::new(area_size, 1..=20));
    ui.label("Offset:");
    ui.add(egui::Slider::new(offset, 0..=20));
}

fn background_editor_ui(
    selected_bg: &mut usize,
    selected_layer: &mut usize,
    backgrounds: &mut IndexMap<BackgroundId, RawBGData>,
    ui: &mut egui::Ui,
) {
    let response = egui::ComboBox::from_id_source("background_id")
        .width(150.)
        .show_index(ui, selected_bg, backgrounds.len(), |index| {
            let (BackgroundId(name), _) = backgrounds.get_index(index).unwrap();
            name.to_owned()
        });
    if response.changed() {
        *selected_layer = 0;
    }

    ui.separator();

    let (_, raw_background) = backgrounds.get_index_mut(*selected_bg).unwrap();

    ui.label("Layers:");

    for (layer_index, layer) in raw_background.texture.0.layers.iter().enumerate() {
        ui.add_enabled_ui(layer_index != *selected_layer, |ui| {
            if ui.button(&layer.texture).clicked() {
                *selected_layer = layer_index;
            }
        });
    }

    ui.separator();

    let layer = &mut raw_background.texture.0.layers[*selected_layer];

    ui.label("Move Factor:");
    ui.add(egui::DragValue::new(&mut layer.move_factor).speed(0.01));
    ui.checkbox(&mut layer.is_looping, "Is Looping");
    ui.label("Offset:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut layer.offset.x));
        ui.add(egui::DragValue::new(&mut layer.offset.y));
    });

    ui.separator();
    if ui.button("Save").clicked() {
        let file = File::create(background::DATA_FILE_PATH).unwrap();
        serde_json_pretty::to_writer(file, backgrounds).unwrap();
        exit(0);
    }
}
