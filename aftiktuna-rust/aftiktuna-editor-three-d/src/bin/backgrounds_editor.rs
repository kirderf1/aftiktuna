use aftiktuna::asset::background::{self, BGData};
use aftiktuna::asset::color;
use aftiktuna::asset::location::DoorType;
use aftiktuna::asset::model::ModelAccess;
use aftiktuna::asset::placement::Positioner;
use aftiktuna::core::Species;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{DialogueExpression, ModelId};
use aftiktuna::core::position::Coord;
use aftiktuna::view::area::ObjectProperties;
use aftiktuna_three_d::asset::{CachedLoader, LazilyLoadedModels};
use aftiktuna_three_d::render::{self, RenderProperties};
use indexmap::IndexMap;
use std::fs::File;
use three_d::egui;

const SIDE_PANEL_WIDTH: u32 = 200;

const SIZE: (u32, u32) = (
    aftiktuna_three_d::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    aftiktuna_three_d::WINDOW_HEIGHT as u32,
);

fn main() {
    let mut backgrounds = background::load_index_map_backgrounds().unwrap();
    let mut selected_bg = 0;
    let mut selected_layer = 0;

    let mut example_content_type = ExampleContentType::None;
    let mut area_size = 5;
    let mut offset = 0;
    let mut camera = aftiktuna_three_d::Camera::default();

    let window = three_d::Window::new(three_d::WindowSettings {
        title: "Aftiktuna: Backgrounds Editor".to_string(),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let mut gui = three_d::GUI::new(&window.gl());
    let mut texture_loader = CachedLoader::new(window.gl());
    let mut models = LazilyLoadedModels::new(window.gl()).unwrap();

    window.render_loop(move |mut frame_input| {
        let mut save = false;

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                side_panel(egui_context, |ui| {
                    display_parameters_ui(
                        &mut area_size,
                        &mut offset,
                        &mut example_content_type,
                        ui,
                    );

                    ui.separator();

                    save = backgrounds_editor_ui(
                        &mut selected_bg,
                        &mut selected_layer,
                        &mut backgrounds,
                        ui,
                    );
                })
            },
        );

        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(area_size);

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: 0,
            width: (frame_input.device_pixel_ratio * f32::from(aftiktuna_three_d::WINDOW_WIDTH))
                as u32,
            height: (frame_input.device_pixel_ratio * f32::from(aftiktuna_three_d::WINDOW_HEIGHT))
                as u32,
        };

        let (_, raw_background) = backgrounds.get_index(selected_bg).unwrap();
        if let Ok(loaded_background) = raw_background.load(&mut texture_loader) {
            let render_camera = render::get_render_camera(&camera, render_viewport);

            let background_objects = render::render_objects_for_primary_background(
                &loaded_background,
                offset,
                camera.camera_x,
                &[],
                &frame_input.context,
            );
            render::draw_in_order(&background_objects, &render_camera, &screen);

            draw_example_content(
                example_content_type,
                area_size,
                render_camera,
                &frame_input,
                &mut models,
            );
        }

        screen.write(|| gui.render()).unwrap();

        if save {
            let file = File::create(background::DATA_FILE_PATH).unwrap();
            serde_json_pretty::to_writer(file, &backgrounds).unwrap();

            three_d::FrameOutput {
                exit: true,
                ..Default::default()
            }
        } else {
            three_d::FrameOutput::default()
        }
    });
}

fn side_panel(ctx: &egui::Context, panel_contents: impl FnOnce(&mut egui::Ui)) {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(SIDE_PANEL_WIDTH as f32)
        .show(ctx, panel_contents);
}

fn display_parameters_ui(
    area_size: &mut Coord,
    offset: &mut i32,
    current_type: &mut ExampleContentType,
    ui: &mut egui::Ui,
) {
    ui.label("Area Size:");
    ui.add(egui::Slider::new(area_size, 1..=20));
    ui.label("Offset:");
    ui.add(egui::Slider::new(offset, -10..=10));
    egui::ComboBox::from_label("Content")
        .selected_text(format!("{current_type:?}"))
        .show_ui(ui, |ui| {
            for &selectable_type in ExampleContentType::variants() {
                ui.selectable_value(
                    current_type,
                    selectable_type,
                    format!("{selectable_type:?}"),
                );
            }
        });
}

fn backgrounds_editor_ui(
    selected_bg: &mut usize,
    selected_layer: &mut usize,
    backgrounds: &mut IndexMap<BackgroundId, BGData<String>>,
    ui: &mut egui::Ui,
) -> bool {
    let response = egui::ComboBox::from_id_salt("background_id")
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

    aftiktuna_editor_three_d::background_layer_list_editor(
        ui,
        selected_layer,
        &mut raw_background.primary.0.layers,
    );

    ui.separator();
    ui.button("Save").clicked()
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

fn draw_example_content(
    example_content_type: ExampleContentType,
    area_size: Coord,
    camera: three_d::Camera,
    frame_input: &three_d::FrameInput,
    models: &mut LazilyLoadedModels,
) {
    let mut positioner = Positioner::default();
    let mut draw_model = move |coord: Coord, model_id: &ModelId| {
        let model = models.lookup_model(model_id);
        let pos = positioner.position_object(coord, model);
        let objects = render::get_render_objects_for_entity_with_color(
            model,
            pos.into(),
            RenderProperties {
                object: &ObjectProperties::default(),
                aftik_color: color::DEFAULT_COLOR,
                expression: DialogueExpression::default(),
            },
            frame_input.accumulated_time as f32,
            &frame_input.context,
        );
        render::draw_in_order(&objects, &camera, &frame_input.screen());
    };

    match example_content_type {
        ExampleContentType::None => {}
        ExampleContentType::Doors => {
            let door = DoorType::Door.into();
            let ship_exit = DoorType::Doorway.into();
            for coord in 0..area_size {
                draw_model(coord, if coord % 2 == 0 { &door } else { &ship_exit });
            }
        }
        ExampleContentType::Paths => {
            let path = DoorType::Path.into();
            for coord in 0..area_size {
                draw_model(coord, &path);
            }
        }
        ExampleContentType::BigObjectsOnEdges => {
            let frog = Species::VoraciousFrog.model_id();
            draw_model(0, &frog);
            if area_size > 1 {
                draw_model(area_size - 1, &frog);
            }
            if area_size > 2 {
                let azureclops = Species::Azureclops.model_id();
                draw_model(area_size / 2, &azureclops);
            }
        }
    }
}
