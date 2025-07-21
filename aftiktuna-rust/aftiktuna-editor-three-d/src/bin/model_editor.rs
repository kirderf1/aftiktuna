use aftiktuna::asset::color::{AftikColorData, ColorSource, RGBColor};
use aftiktuna::asset::model::{self, Model};
use aftiktuna::asset::placement::Positioner;
use aftiktuna::asset::{TextureLoader, background};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::view::area::RenderProperties;
use aftiktuna_three_d::asset::CachedLoader;
use aftiktuna_three_d::render;
use std::fs::{self, File};
use three_d::{Texture2DRef, egui};

const SIDE_PANEL_WIDTH: u32 = 200;

const SIZE: (u32, u32) = (
    aftiktuna_three_d::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    aftiktuna_three_d::WINDOW_HEIGHT as u32,
);

fn main() {
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
    let mut group_size = 3;
    let mut area_size = 7;

    let mut camera = aftiktuna_three_d::Camera::default();

    let window = three_d::Window::new(three_d::WindowSettings {
        title: "Aftiktuna: Model Editor".to_string(),
        min_size: SIZE,
        max_size: Some(SIZE),
        ..Default::default()
    })
    .unwrap();

    let mut gui = three_d::GUI::new(&window.gl());
    let mut texture_loader = CachedLoader::new(window.gl());

    let aftik_model = model::load_raw_model_from_path(ModelId::aftik().file_path())
        .unwrap()
        .load(&mut texture_loader)
        .unwrap();
    let background = background::load_raw_backgrounds()
        .unwrap()
        .get(&BackgroundId::new("forest"))
        .unwrap()
        .load(&mut texture_loader)
        .unwrap();

    window.render_loop(move |mut frame_input| {
        let mut save = false;

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |egui_context| {
                save |= side_panel(
                    egui_context,
                    &mut selected_model,
                    &mut selected_layer,
                    &mut group_size,
                    &mut texture_loader,
                );
            },
        );

        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(area_size);

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: 0,
            width: aftiktuna_three_d::WINDOW_WIDTH.into(),
            height: aftiktuna_three_d::WINDOW_HEIGHT.into(),
        };

        let model = selected_model.load(&mut texture_loader).unwrap();
        let render_camera = render::get_render_camera(&camera, render_viewport);

        let background_objects = render::render_objects_for_primary_background(
            &background,
            0,
            camera.camera_x,
            &[],
            &frame_input.context,
        );
        render::draw_in_order(&background_objects, &render_camera, &screen);

        area_size = draw_examples(
            &model,
            group_size,
            &aftik_model,
            &render_camera,
            &frame_input,
        );

        screen.write(|| gui.render()).unwrap();

        if save {
            let file = File::create(&path).unwrap();
            serde_json_pretty::to_writer(file, &selected_model).unwrap();

            three_d::FrameOutput {
                exit: true,
                ..Default::default()
            }
        } else {
            three_d::FrameOutput::default()
        }
    });
}

fn side_panel(
    ctx: &egui::Context,
    model: &mut Model<String>,
    selected_layer: &mut usize,
    group_size: &mut u16,
    textures: &mut CachedLoader,
) -> bool {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(SIDE_PANEL_WIDTH as f32)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    model_editor_ui(ui, model, selected_layer, group_size, textures)
                })
                .inner
        })
        .inner
}

fn model_editor_ui(
    ui: &mut egui::Ui,
    model: &mut Model<String>,
    selected_layer: &mut usize,
    group_size: &mut u16,
    textures: &mut CachedLoader,
) -> bool {
    if model.has_x_displacement || model.z_displacement != 0 {
        ui.label("Shown count:");
        ui.add(egui::Slider::new(group_size, 1..=20));
        ui.separator();
    }

    ui.label("Wield offset:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut model.wield_offset.0));
        ui.add(egui::DragValue::new(&mut model.wield_offset.1));
    });
    if ui.button("Clear Offset").clicked() {
        model.wield_offset = (0, 0);
    }

    ui.label("Z-offset:");
    ui.add(egui::DragValue::new(&mut model.z_offset));

    ui.checkbox(&mut model.fixed_orientation, "Fixed Direction");

    ui.checkbox(&mut model.has_x_displacement, "X-displacement");
    ui.label("Z-displacement:");
    ui.add(egui::DragValue::new(&mut model.z_displacement));

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
                ui.selectable_value(&mut layer.color, color, format!("{color:?}"));
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

    ui.label("Offset:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut layer.positioning.offset.0));
        ui.add(egui::DragValue::new(&mut layer.positioning.offset.1));
    });

    ui.separator();

    ui.label("Anchor:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut layer.positioning.anchor.0));
        ui.add(egui::DragValue::new(&mut layer.positioning.anchor.1));
    });
    ui.label("Rotation:");
    ui.add(egui::Slider::new(
        &mut layer.positioning.rotation.0,
        -180.0..=180.0,
    ));
    ui.add(egui::Slider::new(
        &mut layer.positioning.rotation.1,
        -180.0..=180.0,
    ));

    ui.horizontal(|ui| {
        ui.label("Animation time:");
        ui.add(
            egui::DragValue::new(&mut layer.positioning.animation_length)
                .speed(0.1)
                .range(0.0..=f32::INFINITY),
        );
    });

    ui.separator();

    ui.button("Save").clicked()
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

const DEFAULT_AFTIK_COLOR: AftikColorData = AftikColorData {
    primary_color: RGBColor::new(148, 216, 0),
    secondary_color: RGBColor::new(255, 238, 153),
};

fn draw_examples(
    model: &Model<Texture2DRef>,
    group_size: u16,
    aftik_model: &Model<Texture2DRef>,
    camera: &three_d::Camera,
    frame_input: &three_d::FrameInput,
) -> Coord {
    let time = frame_input.accumulated_time as f32;
    let context = &frame_input.context;
    let mut positioner = Positioner::default();
    let mut next_coord = 0;
    let mut get_and_move_coord = || {
        let coord = next_coord;
        next_coord += 2;
        coord
    };
    let mut objects = Vec::new();

    bidirectional(|direction| {
        objects.extend(render::get_render_objects_for_entity_with_color(
            model,
            positioner
                .position_object(get_and_move_coord(), model)
                .into(),
            DEFAULT_AFTIK_COLOR,
            &RenderProperties {
                direction,
                ..Default::default()
            },
            time,
            context,
        ));
    });

    if model.has_x_displacement || model.z_displacement != 0 {
        let coord = get_and_move_coord();
        objects.extend(render::get_render_objects_for_entity_with_color(
            model,
            positioner.position_object(coord, model).into(),
            DEFAULT_AFTIK_COLOR,
            &RenderProperties {
                ..Default::default()
            },
            time,
            context,
        ));
        objects.extend(render::get_render_objects_for_entity_with_color(
            aftik_model,
            positioner.position_object(coord, aftik_model).into(),
            DEFAULT_AFTIK_COLOR,
            &RenderProperties {
                ..Default::default()
            },
            time,
            context,
        ));

        let coord = get_and_move_coord();
        objects.extend(render::get_render_objects_for_entity_with_color(
            aftik_model,
            positioner.position_object(coord, aftik_model).into(),
            DEFAULT_AFTIK_COLOR,
            &RenderProperties {
                ..Default::default()
            },
            time,
            context,
        ));
        objects.extend(render::get_render_objects_for_entity_with_color(
            model,
            positioner.position_object(coord, model).into(),
            DEFAULT_AFTIK_COLOR,
            &RenderProperties {
                ..Default::default()
            },
            time,
            context,
        ));

        let coord = get_and_move_coord();
        for (pos, _) in positioner.position_groups_from_offsets(
            model.group_placement.position(group_size),
            coord,
            model,
        ) {
            objects.extend(render::get_render_objects_for_entity_with_color(
                model,
                pos.into(),
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    ..Default::default()
                },
                time,
                context,
            ));
        }
    } else {
        bidirectional(|direction| {
            let coord = get_and_move_coord();
            objects.extend(render::get_render_objects_for_entity_with_color(
                model,
                positioner.position_object(coord, model).into(),
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                time,
                context,
            ));
            objects.extend(render::get_render_objects_for_entity_with_color(
                aftik_model,
                positioner.position_object(coord, aftik_model).into(),
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                time,
                context,
            ));
        })
    }

    if model
        .layers
        .iter()
        .any(|layer| layer.conditions.if_cut.is_some())
    {
        bidirectional(|direction| {
            objects.extend(render::get_render_objects_for_entity_with_color(
                model,
                positioner
                    .position_object(get_and_move_coord(), model)
                    .into(),
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    is_cut: true,
                    ..Default::default()
                },
                time,
                context,
            ));
        });
    }

    if model
        .layers
        .iter()
        .any(|layer| layer.conditions.if_hurt.is_some())
    {
        bidirectional(|direction| {
            objects.extend(render::get_render_objects_for_entity_with_color(
                model,
                positioner
                    .position_object(get_and_move_coord(), model)
                    .into(),
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    is_badly_hurt: true,
                    ..Default::default()
                },
                time,
                context,
            ));
        });
    }

    if model
        .layers
        .iter()
        .any(|layer| layer.conditions.if_alive.is_some())
    {
        bidirectional(|direction| {
            objects.extend(render::get_render_objects_for_entity_with_color(
                model,
                positioner
                    .position_object(get_and_move_coord(), model)
                    .into(),
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    is_alive: false,
                    ..Default::default()
                },
                time,
                context,
            ));
        });
    }

    if model.wield_offset != (0, 0) {
        bidirectional(|direction| {
            let pos = positioner
                .position_object(get_and_move_coord(), aftik_model)
                .into();
            objects.extend(render::get_render_objects_for_entity_with_color(
                aftik_model,
                pos,
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                time,
                context,
            ));
            let direction_mod = match direction {
                Direction::Left => -1,
                Direction::Right => 1,
            };
            let offset = three_d::vec2(
                f32::from(direction_mod * model.wield_offset.0),
                f32::from(-model.wield_offset.1),
            );
            objects.extend(render::get_render_objects_for_entity_with_color(
                model,
                pos + offset,
                DEFAULT_AFTIK_COLOR,
                &RenderProperties {
                    direction,
                    ..Default::default()
                },
                time,
                context,
            ));
        });
    }

    render::draw_in_order(&objects, camera, &frame_input.screen());

    next_coord - 1
}

fn bidirectional(mut closure: impl FnMut(Direction)) {
    closure(Direction::Right);
    closure(Direction::Left);
}
