use crate::asset::{Assets, LazilyLoadedModels};
use aftiktuna::asset;
use aftiktuna::asset::background::{BGData, PortraitBGData};
use aftiktuna::asset::color::{AftikColorData, RGBColor};
use aftiktuna::asset::model::{Model, TextureLayer};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId};
use aftiktuna::core::position::{Coord, Direction};
use aftiktuna::view::area::{ObjectRenderData, RenderData, RenderProperties};
use aftiktuna::view::Frame;
use std::collections::HashMap;
use three_d::Object;

pub fn render_frame(
    frame: &Frame,
    camera: &super::Camera,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    match frame {
        Frame::Introduction | Frame::LocationChoice(_) | Frame::Error(_) => {
            let background_objects = render_objects_for_primary_background(
                assets
                    .backgrounds
                    .get_or_default(&BackgroundId::location_choice()),
                0,
                0.,
                &frame_input.context,
            );

            let render_camera = super::default_render_camera(frame_input.viewport);
            draw_in_order(&background_objects, render_camera, screen);
        }
        Frame::AreaView { render_data, .. } => {
            draw_area_view(render_data, camera, screen, frame_input, assets);
        }
        Frame::Dialogue { data, .. } => {
            draw_secondary_background(
                assets.backgrounds.get_or_default(&data.background),
                screen,
                frame_input,
            );

            let x = match data.direction {
                Direction::Left => super::WINDOW_WIDTH_F - 300.,
                Direction::Right => 300.,
            };
            let objects = get_render_objects_for_entity(
                assets.models.lookup_model(&ModelId::portrait()),
                three_d::vec2(x, 0.),
                &RenderProperties {
                    direction: data.direction,
                    aftik_color: data.color.clone(),
                    is_badly_hurt: data.is_badly_hurt,
                    ..RenderProperties::default()
                },
                &mut assets.aftik_colors,
                &frame_input.context,
            );
            let render_camera = super::default_render_camera(frame_input.viewport);
            draw_in_order(&objects, render_camera, screen);
        }
        Frame::StoreView { view, .. } => {
            draw_secondary_background(
                assets.backgrounds.get_or_default(&view.background),
                screen,
                frame_input,
            );

            let objects = get_render_objects_for_entity(
                assets.models.lookup_model(&ModelId::portrait()),
                three_d::vec2(super::WINDOW_WIDTH_F - 200., 0.),
                &RenderProperties {
                    direction: Direction::Left,
                    aftik_color: view.shopkeeper_color.clone(),
                    ..RenderProperties::default()
                },
                &mut assets.aftik_colors,
                &frame_input.context,
            );
            let render_camera = super::default_render_camera(frame_input.viewport);
            draw_in_order(&objects, render_camera, screen);
        }
        Frame::Ending { stop_type } => {
            let (r, g, b) = match stop_type {
                aftiktuna::StopType::Win => (0.78, 0.78, 0.78),
                aftiktuna::StopType::Lose => (0., 0., 0.),
            };
            screen.clear(three_d::ClearState::color(r, g, b, 1.));
        }
    }
}

fn draw_area_view(
    render_data: &RenderData,
    camera: &super::Camera,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
    assets: &mut Assets,
) {
    let background_objects = render_objects_for_primary_background(
        assets.backgrounds.get_or_default(&render_data.background),
        render_data.background_offset.unwrap_or(0),
        camera.camera_x,
        &frame_input.context,
    );
    let entity_objects = position_objects(&render_data.objects, &mut assets.models)
        .into_iter()
        .flat_map(|(pos, object)| {
            get_render_objects_for_entity(
                assets.models.lookup_model(&object.model_id),
                pos,
                &object.properties,
                &mut assets.aftik_colors,
                &frame_input.context,
            )
        })
        .collect::<Vec<_>>();

    let render_camera = camera.get_render_camera(frame_input.viewport);
    screen
        .write::<three_d::RendererError>(|| {
            for object in background_objects {
                object.render(&render_camera, &[]);
            }
            for object in entity_objects {
                object.render(&render_camera, &[]);
            }
            Ok(())
        })
        .unwrap();
}

fn render_objects_for_primary_background(
    background: &BGData<three_d::Texture2DRef>,
    background_offset: Coord,
    camera_x: f32,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    let offset = background_offset as f32 * 120.;

    background
        .primary
        .0
        .layers
        .iter()
        .flat_map(|layer| {
            let width = layer.texture.width() as f32;
            let height = layer.texture.height() as f32;
            let layer_x = f32::from(layer.offset.x) + camera_x * (1. - layer.move_factor) - offset;
            let layer_y = f32::from(layer.offset.y);
            let material = three_d::ColorMaterial {
                texture: Some(layer.texture.clone()),
                render_states: three_d::RenderStates {
                    write_mask: three_d::WriteMask::COLOR,
                    blend: three_d::Blend::STANDARD_TRANSPARENCY,
                    ..Default::default()
                },
                ..Default::default()
            };

            fn rect(
                x: f32,
                y: f32,
                width: f32,
                height: f32,
                context: &three_d::Context,
            ) -> three_d::Rectangle {
                three_d::Rectangle::new(
                    context,
                    three_d::vec2(x + width / 2., y + height / 2.),
                    three_d::degrees(0.),
                    width,
                    height,
                )
            }

            if layer.is_looping {
                let repeat_start = f32::floor((camera_x - layer_x) / width) as i16;
                let repeat_end = f32::floor((camera_x + 800. - layer_x) / width) as i16;
                (repeat_start..=repeat_end)
                    .map(|repeat_index| {
                        three_d::Gm::new(
                            rect(
                                layer_x + width * f32::from(repeat_index),
                                layer_y,
                                width,
                                height,
                                context,
                            ),
                            material.clone(),
                        )
                    })
                    .collect()
            } else {
                vec![three_d::Gm::new(
                    rect(layer_x, layer_y, width, height, context),
                    material,
                )]
            }
        })
        .collect()
}

fn draw_secondary_background(
    background: &BGData<three_d::Texture2DRef>,
    screen: &three_d::RenderTarget<'_>,
    frame_input: &three_d::FrameInput,
) {
    match &background.portrait {
        &PortraitBGData::Color([r, g, b]) => {
            screen.clear(three_d::ClearState::color(
                f32::from(r) / 255.,
                f32::from(g) / 255.,
                f32::from(b) / 255.,
                1.,
            ));
        }
        PortraitBGData::Texture(texture) => {
            let background_object = three_d::Gm::new(
                three_d::Rectangle::new(
                    &frame_input.context,
                    three_d::vec2(crate::WINDOW_WIDTH_F / 2., crate::WINDOW_HEIGHT_F / 2.),
                    three_d::degrees(0.),
                    crate::WINDOW_WIDTH_F,
                    crate::WINDOW_HEIGHT_F,
                ),
                three_d::ColorMaterial {
                    texture: Some(texture.clone()),
                    render_states: three_d::RenderStates {
                        write_mask: three_d::WriteMask::COLOR,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );

            let render_camera = super::default_render_camera(frame_input.viewport);
            screen.render(&render_camera, [background_object], &[]);
        }
    };
}

fn get_render_objects_for_entity(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    aftik_colors: &mut HashMap<AftikColorId, AftikColorData>,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    let aftik_color = properties
        .aftik_color
        .as_ref()
        .map_or(asset::color::DEFAULT_COLOR, |aftik_color| {
            lookup_or_log_aftik_color(aftik_color, aftik_colors)
        });
    model
        .layers
        .iter()
        .flat_map(|layer| get_render_object_for_layer(layer, pos, properties, aftik_color, context))
        .collect()
}

fn lookup_or_log_aftik_color(
    color_id: &AftikColorId,
    aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
) -> AftikColorData {
    aftik_colors_map.get(color_id).copied().unwrap_or_else(|| {
        eprintln!("Missing aftik color data for color {color_id:?}!");
        aftik_colors_map.insert(color_id.clone(), asset::color::DEFAULT_COLOR);
        asset::color::DEFAULT_COLOR
    })
}

fn get_render_object_for_layer(
    layer: &TextureLayer<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    aftik_color: AftikColorData,
    context: &three_d::Context,
) -> Option<impl three_d::Object> {
    if !layer.conditions.meets_conditions(properties) {
        return None;
    }

    let (width, height) = layer
        .positioning
        .size
        .map(|(width, height)| (f32::from(width), f32::from(height)))
        .unwrap_or_else(|| (layer.texture.width() as f32, layer.texture.height() as f32));
    let direction = if !layer.positioning.fixed && properties.direction == Direction::Left {
        -1.
    } else {
        1.
    };
    let rectangle = three_d::Rectangle::new(
        context,
        three_d::vec2(
            pos.x,
            pos.y + height / 2. - f32::from(layer.positioning.y_offset),
        ),
        three_d::degrees(0.),
        width * direction,
        height,
    );

    let color = layer.color.get_color(&aftik_color);
    let material = UnalteredColorMaterial(
        three_d::ColorMaterial {
            texture: Some(layer.texture.clone()),
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    );

    Some(three_d::Gm::new(rectangle, material))
}

pub struct UnalteredColorMaterial(pub three_d::ColorMaterial, pub RGBColor);

impl three_d::Material for UnalteredColorMaterial {
    fn id(&self) -> three_d::EffectMaterialId {
        self.0.id()
    }

    fn fragment_shader_source(&self, lights: &[&dyn three_d::Light]) -> String {
        self.0.fragment_shader_source(lights)
    }

    fn use_uniforms(
        &self,
        program: &three_d::Program,
        viewer: &dyn three_d::Viewer,
        _lights: &[&dyn three_d::Light],
    ) {
        viewer.color_mapping().use_uniforms(program);
        let color = three_d::vec4(
            f32::from(self.1.r) / 255.,
            f32::from(self.1.g) / 255.,
            f32::from(self.1.b) / 255.,
            1.,
        );
        program.use_uniform("surfaceColor", color);
        if let Some(ref tex) = self.0.texture {
            program.use_uniform("textureTransformation", tex.transformation);
            program.use_texture("tex", tex);
        }
    }

    fn render_states(&self) -> three_d::RenderStates {
        self.0.render_states()
    }

    fn material_type(&self) -> three_d::MaterialType {
        self.0.material_type()
    }
}

fn draw_in_order(
    objects: &[impl three_d::Object],
    camera: three_d::Camera,
    screen: &three_d::RenderTarget<'_>,
) {
    screen
        .write::<three_d::RendererError>(|| {
            for object in objects {
                object.render(&camera, &[]);
            }
            Ok(())
        })
        .unwrap();
}

pub fn position_objects<'a>(
    objects: &'a Vec<ObjectRenderData>,
    models: &mut LazilyLoadedModels,
) -> Vec<(three_d::Vec2, &'a ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut positioner = Positioner::new();

    for data in objects {
        let pos = positioner.position_object(
            data.coord,
            models.lookup_model(&data.model_id).is_displacing(),
        );

        positioned_objects.push((pos, data));
    }
    positioned_objects
}

fn position_from_coord(coord: Coord, count: i32) -> three_d::Vec2 {
    three_d::vec2(
        crate::coord_to_center_x(coord) - count as f32 * 15.,
        (190 - count * 15) as f32,
    )
}

#[derive(Default)]
struct Positioner {
    coord_counts: HashMap<Coord, i32>,
}

impl Positioner {
    pub fn new() -> Self {
        Self::default()
    }

    fn position_object(&mut self, coord: Coord, is_displacing: bool) -> three_d::Vec2 {
        if is_displacing {
            let count_ref = self.coord_counts.entry(coord).or_insert(0);
            let count = *count_ref;
            *count_ref = count + 1;
            position_from_coord(coord, count)
        } else {
            position_from_coord(coord, 0)
        }
    }
}
