use aftiktuna::asset::background::{self, BGData};
use aftiktuna::asset::color;
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::placement::Positioner;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::Coord;
use aftiktuna::view::area::RenderProperties;
use aftiktuna_macroquad::camera::HorizontalDraggableCamera;
use aftiktuna_macroquad::egui::EguiWrapper;
use aftiktuna_macroquad::texture::background as mq_background;
use aftiktuna_macroquad::texture::{model, CachedTextures, LazilyLoadedModels};
use indexmap::IndexMap;
use macroquad::color as mq_color;
use macroquad::window::{self, Conf};
use std::fs::File;
use std::process::exit;

fn config() -> Conf {
    Conf {
        window_title: "Aftiktuna Model Editor".to_string(),
        window_width: 1000,
        window_height: 600,
        window_resizable: false,
        icon: Some(aftiktuna_macroquad::logo()),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut textures = CachedTextures::default();
    let mut models = model::prepare().unwrap();

    let mut backgrounds = background::load_index_map_backgrounds().unwrap();
    let mut selected_bg = 0;
    let mut selected_layer = 0;

    let mut example_content_type = ExampleContentType::None;
    let mut area_size = 5;
    let mut offset = 0;
    let mut camera = HorizontalDraggableCamera::centered_on_position(0, area_size);
    camera.set_default_size_viewport(0, 0);

    let mut egui = EguiWrapper::init();

    loop {
        window::clear_background(mq_color::BLACK);

        let mut is_mouse_over_panel = false;
        egui.ui(|ctx| {
            is_mouse_over_panel |= side_panel(ctx, |ui| {
                display_parameters_ui(&mut area_size, &mut offset, &mut example_content_type, ui);

                ui.separator();

                background_editor_ui(&mut selected_bg, &mut selected_layer, &mut backgrounds, ui);
            });
        });

        camera.clamp(area_size);
        camera.handle_drag(area_size, !is_mouse_over_panel);

        macroquad::camera::set_camera(&camera);

        let (_, raw_background) = backgrounds.get_index(selected_bg).unwrap();
        let loaded_background = raw_background.load(&mut textures).unwrap();
        mq_background::draw_primary(&loaded_background.primary, offset, &camera);

        draw_example_content(example_content_type, area_size, &mut models);

        macroquad::camera::set_default_camera();

        egui.draw();
        window::next_frame().await;
    }
}

fn draw_example_content(
    example_content_type: ExampleContentType,
    area_size: Coord,
    models: &mut LazilyLoadedModels,
) {
    let mut positioner = Positioner::default();
    let mut draw_model = move |coord: Coord, model_id: &ModelId| {
        let model = models.lookup_model(model_id);
        let pos = positioner.position_object(coord, model);
        model::draw_model(
            model,
            aftiktuna_macroquad::to_vec2(pos),
            false,
            &RenderProperties::default(),
            &color::DEFAULT_COLOR,
        );
    };

    match example_content_type {
        ExampleContentType::None => {}
        ExampleContentType::Doors => {
            let door = ModelId::new("door");
            let ship_exit = ModelId::new("doorway");
            for coord in 0..area_size {
                draw_model(coord, if coord % 2 == 0 { &door } else { &ship_exit });
            }
        }
        ExampleContentType::Paths => {
            let path = ModelId::new("path");
            for coord in 0..area_size {
                draw_model(coord, &path);
            }
        }
        ExampleContentType::BigObjectsOnEdges => {
            let frog = ModelId::creature("voracious_frog");
            draw_model(0, &frog);
            if area_size > 1 {
                draw_model(area_size - 1, &frog);
            }
            if area_size > 2 {
                let azureclops = ModelId::creature("azureclops");
                draw_model(area_size / 2, &azureclops);
            }
        }
    }
}

fn side_panel(ctx: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) -> bool {
    let response = egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(200.)
        .show(ctx, panel_contents);
    response.response.contains_pointer()
}

fn display_parameters_ui(
    area_size: &mut Coord,
    offset: &mut Coord,
    current_type: &mut ExampleContentType,
    ui: &mut egui::Ui,
) {
    ui.label("Area Size:");
    ui.add(egui::Slider::new(area_size, 1..=20));
    ui.label("Offset:");
    ui.add(egui::Slider::new(offset, 0..=20));
    egui::ComboBox::from_label("Content")
        .selected_text(format!("{:?}", current_type))
        .show_ui(ui, |ui| {
            for &selectable_type in ExampleContentType::variants() {
                ui.selectable_value(
                    current_type,
                    selectable_type,
                    format!("{:?}", selectable_type),
                );
            }
        });
}

fn background_editor_ui(
    selected_bg: &mut usize,
    selected_layer: &mut usize,
    backgrounds: &mut IndexMap<BackgroundId, BGData<String>>,
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

    for (layer_index, layer) in raw_background.primary.0.layers.iter().enumerate() {
        ui.add_enabled_ui(layer_index != *selected_layer, |ui| {
            if ui.button(&layer.texture).clicked() {
                *selected_layer = layer_index;
            }
        });
    }

    ui.separator();

    let layer = &mut raw_background.primary.0.layers[*selected_layer];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExampleContentType {
    None,
    Doors,
    Paths,
    BigObjectsOnEdges,
}

impl ExampleContentType {
    fn variants() -> &'static [Self] {
        use ExampleContentType::*;
        &[None, Doors, Paths, BigObjectsOnEdges]
    }
}
