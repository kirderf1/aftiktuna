use std::fs::File;
use std::mem::take;
use std::process::exit;

use aftiktuna::core::{AftikColorId, ModelId};
use aftiktuna::macroquad_interface;
use aftiktuna::macroquad_interface::texture::model::Model;
use aftiktuna::macroquad_interface::texture::{model, AftikColorData, RGBColor};
use aftiktuna::view::area::RenderProperties;
use egui_macroquad::egui;
use egui_macroquad::macroquad::math::Vec2;
use egui_macroquad::macroquad::window::{self, Conf};
use egui_macroquad::macroquad::{self, color};
use indexmap::IndexMap;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna Color Editor".to_string(),
        window_width: 600,
        window_height: 500,
        window_resizable: false,
        icon: Some(macroquad_interface::logo()),
        ..Default::default()
    }
}

type AftikColorMap = IndexMap<AftikColorId, AftikColorData>;

#[macroquad::main(config)]
async fn main() {
    let mut aftik_colors = load_aftik_colors_ordered();

    let aftik_model = model::load_model(&ModelId::aftik()).expect("Unable to load aftik model");

    let mut selected_index = 0;
    let mut new_color_name = String::new();

    if aftik_colors.is_empty() {
        init_new_color(
            AftikColorId::new("mint"),
            &mut selected_index,
            &mut aftik_colors,
        );
    }

    loop {
        window::clear_background(color::LIGHTGRAY);

        egui_macroquad::ui(|ctx| {
            side_panel(
                ctx,
                &mut selected_index,
                &mut new_color_name,
                &mut aftik_colors,
            );
        });

        let (_, aftik_color_data) = aftik_colors.get_index(selected_index).unwrap();
        draw_examples(aftik_color_data, &aftik_model);

        egui_macroquad::draw();
        window::next_frame().await;
    }
}

fn load_aftik_colors_ordered() -> AftikColorMap {
    let file = File::open(macroquad_interface::texture::AFTIK_COLORS_PATH)
        .expect("Unable to open aftik color file");
    serde_json::from_reader::<_, IndexMap<_, _>>(file).expect("Unable to load aftik color data")
}

fn draw_examples(aftik_color_data: &AftikColorData, aftik_model: &Model) {
    aftik_model.draw(
        Vec2::new(100., 250.),
        false,
        &RenderProperties {
            is_alive: true,
            is_badly_hurt: false,
            ..Default::default()
        },
        aftik_color_data,
    );
    aftik_model.draw(
        Vec2::new(250., 250.),
        false,
        &RenderProperties {
            is_alive: true,
            is_badly_hurt: true,
            ..Default::default()
        },
        aftik_color_data,
    );
    aftik_model.draw(
        Vec2::new(150., 350.),
        false,
        &RenderProperties {
            is_alive: false,
            ..Default::default()
        },
        aftik_color_data,
    );
}

fn side_panel(
    ctx: &egui::Context,
    selected_index: &mut usize,
    new_color_name: &mut String,
    aftik_colors: &mut AftikColorMap,
) {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(200.)
        .show(ctx, |ui| {
            ui.text_edit_singleline(new_color_name);

            if ui.button("Add").clicked() && !new_color_name.is_empty() {
                init_new_color(
                    AftikColorId(take(new_color_name)),
                    selected_index,
                    aftik_colors,
                );
            }

            egui::ComboBox::from_id_source("color_id").show_index(
                ui,
                selected_index,
                aftik_colors.len(),
                |index| {
                    let (AftikColorId(name), _) = aftik_colors.get_index(index).unwrap();
                    name.to_owned()
                },
            );

            ui.separator();

            let (_, aftik_color_data) = aftik_colors.get_index_mut(*selected_index).unwrap();

            ui.label("Primary color:");
            color_picker(ui, &mut aftik_color_data.primary_color);

            ui.separator();

            ui.label("Secondary color:");
            color_picker(ui, &mut aftik_color_data.secondary_color);

            ui.separator();

            if ui.button("Save").clicked() {
                save_map(aftik_colors);
                exit(0);
            }
        });
}

fn init_new_color(
    new_id: AftikColorId,
    selected_index: &mut usize,
    aftik_colors: &mut AftikColorMap,
) {
    if !aftik_colors.contains_key(&new_id) {
        *selected_index = aftik_colors
            .insert_full(new_id, macroquad_interface::texture::DEFAULT_COLOR)
            .0;
    }
}

fn color_picker(ui: &mut egui::Ui, color: &mut RGBColor) {
    let mut color32 = (*color).into();
    egui::color_picker::color_picker_color32(ui, &mut color32, egui::color_picker::Alpha::Opaque);
    *color = RGBColor::new(color32.r(), color32.g(), color32.b());
}

fn save_map(aftik_colors: &mut AftikColorMap) {
    let file = File::create(macroquad_interface::texture::AFTIK_COLORS_PATH).unwrap();
    serde_json_pretty::to_writer(file, aftik_colors).unwrap();
}
