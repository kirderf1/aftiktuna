use aftiktuna::asset::color::{AftikColorData, RGBColor};
use aftiktuna::asset::model::{self, LayerCondition, Model, ModelAccess, TexturesOrChildren};
use aftiktuna::asset::{TextureLoader, background, placement};
use aftiktuna::core::Species;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{DialogueExpression, ModelId};
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::view::area::{ObjectProperties, ObjectRenderData};
use aftiktuna_three_d::asset::{CachedLoader, LazilyLoadedModels};
use aftiktuna_three_d::dimensions;
use aftiktuna_three_d::render::{self, RenderProperties};
use std::fs::{self, File};
use three_d::{Texture2DRef, egui};

const SIDE_PANEL_WIDTH: u32 = 200;

const SIZE: (u32, u32) = (
    dimensions::WINDOW_WIDTH as u32 + SIDE_PANEL_WIDTH,
    dimensions::WINDOW_HEIGHT as u32,
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

    let selected_model = model::load_raw_model_from_path(&path).unwrap();
    assert!(
        !selected_model.layers.is_empty(),
        "Layers must not be empty"
    );
    let mut editor_data = EditorData {
        model: selected_model,
        selected_layer: 0,
        group_size: 1,
        indoors: false,
        direction: Direction::default(),
        show_alive: true,
        show_hurt: false,
        show_cut: false,
        shown_expression: DialogueExpression::Neutral,
        setting: SettingType::None,
        example_model: Species::Aftik.model_id(),
    };
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

    let mut example_models = LazilyLoadedModels::new(window.gl()).unwrap();
    let backgrounds_map = background::load_raw_backgrounds().unwrap();
    let forest_background = backgrounds_map
        .get(&BackgroundId::new("forest"))
        .unwrap()
        .load(&mut texture_loader)
        .unwrap();
    let indoor_background = backgrounds_map
        .get(&BackgroundId::new("facility_size3"))
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
                save |= side_panel(egui_context, &mut editor_data, &mut texture_loader);
            },
        );

        camera.handle_inputs(&mut frame_input.events);
        camera.clamp(area_size);

        let screen = frame_input.screen();
        screen.clear(three_d::ClearState::color_and_depth(0., 0., 0., 1., 1.));

        let render_viewport = three_d::Viewport {
            x: 0,
            y: 0,
            width: (frame_input.device_pixel_ratio * f32::from(dimensions::WINDOW_WIDTH)) as u32,
            height: (frame_input.device_pixel_ratio * f32::from(dimensions::WINDOW_HEIGHT)) as u32,
        };

        let loaded_model = editor_data.model.load(&mut texture_loader).unwrap();
        let render_camera = render::get_render_camera(&camera, render_viewport);

        let background_objects = render::render_objects_for_primary_background(
            if editor_data.indoors {
                &indoor_background
            } else {
                &forest_background
            },
            0,
            camera.camera_x,
            &[],
            &frame_input.context,
        );
        render::draw_in_order(&background_objects, &render_camera, &screen);

        area_size = draw_examples(
            &editor_data,
            EditorModels {
                editor_model: &loaded_model,
                example_models: &mut example_models,
            },
            &render_camera,
            &frame_input,
        );

        screen.write(|| gui.render()).unwrap();

        if save {
            let file = File::create(&path).unwrap();
            serde_json_pretty::to_writer(file, &editor_data.model).unwrap();

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
    model: Model<String>,
    selected_layer: usize,
    group_size: u16,
    indoors: bool,
    direction: Direction,
    show_alive: bool,
    show_hurt: bool,
    show_cut: bool,
    shown_expression: DialogueExpression,
    setting: SettingType,
    example_model: ModelId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingType {
    None,
    Behind,
    InFront,
    Facing,
}

impl SettingType {
    fn variants() -> &'static [Self] {
        use SettingType::*;
        &[None, Behind, InFront, Facing]
    }
}

fn side_panel(
    ctx: &egui::Context,
    editor_data: &mut EditorData,
    textures: &mut CachedLoader,
) -> bool {
    egui::SidePanel::right("side")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(8.))
        .resizable(false)
        .exact_width(SIDE_PANEL_WIDTH as f32)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| model_editor_ui(ui, editor_data, textures))
                .inner
        })
        .inner
}

fn model_editor_ui(
    ui: &mut egui::Ui,
    editor_data: &mut EditorData,
    textures: &mut CachedLoader,
) -> bool {
    let model = &mut editor_data.model;
    if model.has_x_displacement || model.z_displacement != 0 {
        ui.label("Shown count:");
        ui.add(egui::Slider::new(&mut editor_data.group_size, 1..=20));
    }

    ui.checkbox(&mut editor_data.indoors, "Indoors");
    aftiktuna_editor_three_d::direction_editor(ui, &mut editor_data.direction, "view direction");

    if model.layers.iter().any(|layer| {
        layer
            .condition
            .0
            .iter()
            .any(|condition| matches!(condition, LayerCondition::Alive(_)))
    }) {
        ui.checkbox(&mut editor_data.show_alive, "Show Alive");
    }
    if model.layers.iter().any(|layer| {
        layer
            .condition
            .0
            .iter()
            .any(|condition| matches!(condition, LayerCondition::Hurt(_)))
    }) {
        ui.checkbox(&mut editor_data.show_hurt, "Show Hurt");
    }
    if model.layers.iter().any(|layer| {
        layer
            .condition
            .0
            .iter()
            .any(|condition| matches!(condition, LayerCondition::Cut(_)))
    }) {
        ui.checkbox(&mut editor_data.show_cut, "Show Cut");
    }

    egui::ComboBox::from_label("Expression")
        .selected_text(format!("{:?}", editor_data.shown_expression))
        .show_ui(ui, |ui| {
            for &selectable_type in DialogueExpression::variants() {
                ui.selectable_value(
                    &mut editor_data.shown_expression,
                    selectable_type,
                    format!("{selectable_type:?}"),
                );
            }
        });

    egui::ComboBox::from_label("Setting")
        .selected_text(format!("{:?}", editor_data.setting))
        .show_ui(ui, |ui| {
            for &selectable_type in SettingType::variants() {
                ui.selectable_value(
                    &mut editor_data.setting,
                    selectable_type,
                    format!("{selectable_type:?}"),
                );
            }
        });
    ui.text_edit_singleline(&mut editor_data.example_model.0);

    ui.separator();

    ui.label("Wield offset:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut model.wield_offset.x));
        ui.add(egui::DragValue::new(&mut model.wield_offset.y));
    });
    if ui.button("Clear Offset").clicked() {
        model.wield_offset = Default::default();
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
        ui.add_enabled_ui(layer_index != editor_data.selected_layer, |ui| {
            let name = match &layer.textures_or_children {
                TexturesOrChildren::Texture(colored_textures) => colored_textures.primary_texture(),
                TexturesOrChildren::Children(_) => "Bone",
            };
            if ui.button(name).clicked() {
                editor_data.selected_layer = layer_index;
            }
        });
    }

    let layer = &mut model.layers[editor_data.selected_layer];

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Animation time:");
        ui.add(
            egui::DragValue::new(&mut layer.positioning.animation_length)
                .speed(0.1)
                .range(0.0..=f32::INFINITY),
        );
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
    } else if let TexturesOrChildren::Texture(colored_textures) = &layer.textures_or_children {
        if ui.button("Use Custom Size").clicked() {
            let texture = textures
                .load_texture(model::texture_path(colored_textures.primary_texture()))
                .unwrap();
            layer.positioning.size = Some((texture.width() as i16, texture.height() as i16));
        }
    }

    ui.label("Offset:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut layer.positioning.offset.0.x));
        ui.add(egui::DragValue::new(&mut layer.positioning.offset.0.y));
    });
    if layer.positioning.animation_length != 0. {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut layer.positioning.offset.1.x));
            ui.add(egui::DragValue::new(&mut layer.positioning.offset.1.y));
        });
    } else {
        layer.positioning.offset.1 = layer.positioning.offset.0;
    }

    ui.separator();

    ui.label("Anchor:");
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut layer.positioning.anchor.x));
        ui.add(egui::DragValue::new(&mut layer.positioning.anchor.y));
    });
    ui.label("Rotation:");
    ui.add(egui::Slider::new(
        &mut layer.positioning.rotation.0,
        -45.0..=45.0,
    ));
    if layer.positioning.animation_length != 0. {
        ui.add(egui::Slider::new(
            &mut layer.positioning.rotation.1,
            -45.0..=45.0,
        ));
    } else {
        layer.positioning.rotation.1 = layer.positioning.rotation.0;
    }

    ui.separator();

    ui.button("Save").clicked()
}

const DEFAULT_AFTIK_COLOR: AftikColorData = AftikColorData {
    primary_color: RGBColor::new(148, 216, 0),
    secondary_color: RGBColor::new(255, 238, 153),
};

fn editor_model_id() -> ModelId {
    ModelId::new("editor_model")
}

struct EditorModels<'a> {
    editor_model: &'a Model<Texture2DRef>,
    example_models: &'a mut LazilyLoadedModels,
}

impl ModelAccess<Texture2DRef> for EditorModels<'_> {
    fn lookup_model(&mut self, model_id: &ModelId) -> &Model<Texture2DRef> {
        if model_id == &editor_model_id() {
            self.editor_model
        } else {
            self.example_models.lookup_model(model_id)
        }
    }
}

fn draw_examples(
    editor_data: &EditorData,
    mut model_access: EditorModels,
    camera: &three_d::Camera,
    frame_input: &three_d::FrameInput,
) -> Coord {
    let time = frame_input.accumulated_time as f32;
    let context = &frame_input.context;
    let mut next_coord = 0;
    let mut get_and_move_coord = || {
        let coord = next_coord;
        next_coord += 1;
        coord
    };
    let mut objects = Vec::new();

    fn obj(coord: Coord, model_id: ModelId, properties: ObjectProperties) -> ObjectRenderData {
        ObjectRenderData {
            coord,
            model_id,
            hash: 0,
            is_controlled: false,
            name_data: None,
            wielded_item: None,
            interactions: Vec::default(),
            properties,
        }
    }

    let properties = ObjectProperties {
        direction: editor_data.direction,
        is_cut: editor_data.show_cut,
        is_alive: editor_data.show_alive,
        is_badly_hurt: editor_data.show_hurt,
        expression: editor_data.shown_expression,
        ..Default::default()
    };

    if editor_data.setting == SettingType::Facing && editor_data.direction == Direction::Left {
        let coord = get_and_move_coord();
        objects.push(obj(
            coord,
            editor_data.example_model.clone(),
            ObjectProperties {
                direction: Direction::Right,
                ..Default::default()
            },
        ));
    }

    let coord = get_and_move_coord();

    if editor_data.setting == SettingType::InFront {
        objects.push(obj(
            coord,
            editor_data.example_model.clone(),
            ObjectProperties {
                direction: editor_data.direction,
                ..Default::default()
            },
        ));
    }

    for _ in 1..=editor_data.group_size {
        objects.push(obj(coord, editor_model_id(), properties.clone()));
    }

    if editor_data.setting == SettingType::Behind {
        objects.push(obj(
            coord,
            editor_data.example_model.clone(),
            ObjectProperties {
                direction: editor_data.direction,
                ..Default::default()
            },
        ));
    }

    if editor_data.setting == SettingType::Facing && editor_data.direction == Direction::Right {
        let coord = get_and_move_coord();
        objects.push(obj(
            coord,
            editor_data.example_model.clone(),
            ObjectProperties {
                direction: Direction::Left,
                ..Default::default()
            },
        ));
    }

    if model_access.editor_model.wield_offset != Default::default() {
        objects.push(ObjectRenderData {
            wielded_item: Some(editor_model_id()),
            ..obj(
                coord,
                Species::Aftik.model_id(),
                ObjectProperties {
                    direction: editor_data.direction,
                    ..Default::default()
                },
            )
        });
    }

    let objects = placement::position_objects(&objects, &mut model_access);

    let objects = objects
        .into_iter()
        .flat_map(|(pos, data)| {
            let pos = pos.into();
            let mut objects = render::get_render_objects_for_entity_with_color(
                model_access.lookup_model(&data.model_id),
                pos,
                RenderProperties {
                    object: &data.properties,
                    aftik_color: DEFAULT_AFTIK_COLOR,
                },
                time,
                context,
            );
            if let Some(wielded) = data.wielded_item {
                let model = model_access.lookup_model(&wielded);
                let offset =
                    aftiktuna_three_d::to_vec(model.wield_offset, data.properties.direction.into());
                objects.extend(render::get_render_objects_for_entity_with_color(
                    model,
                    pos + offset,
                    RenderProperties {
                        object: &ObjectProperties {
                            direction: data.properties.direction,
                            ..Default::default()
                        },
                        aftik_color: DEFAULT_AFTIK_COLOR,
                    },
                    time,
                    context,
                ))
            }
            objects
        })
        .collect::<Vec<_>>();

    render::draw_in_order(&objects, camera, &frame_input.screen());

    next_coord
}
