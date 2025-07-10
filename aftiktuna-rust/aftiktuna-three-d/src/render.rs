use aftiktuna::asset::background::{BGData, PortraitBGData};
use aftiktuna::asset::color::{self, AftikColorData};
use aftiktuna::asset::model::{Model, TextureLayer};
use aftiktuna::core::display::AftikColorId;
use aftiktuna::core::position::Direction;
use aftiktuna::view::area::RenderProperties;
use std::collections::HashMap;

pub fn render_objects_for_primary_background(
    background: &BGData<three_d::Texture2DRef>,
    background_offset: i32,
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
            let material = texture_material(&layer.texture);

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

pub fn draw_secondary_background(
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
                texture_material(texture),
            );

            let render_camera = default_render_camera(frame_input.viewport);
            screen.render(&render_camera, [background_object], &[]);
        }
    };
}

pub fn get_render_objects_for_entity(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
    aftik_colors: &mut HashMap<AftikColorId, AftikColorData>,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    let aftik_color = properties
        .aftik_color
        .as_ref()
        .map_or(color::DEFAULT_COLOR, |aftik_color| {
            lookup_or_log_aftik_color(aftik_color, aftik_colors)
        });
    get_render_objects_for_entity_with_color(model, pos, aftik_color, properties, context)
}

fn lookup_or_log_aftik_color(
    color_id: &AftikColorId,
    aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
) -> AftikColorData {
    aftik_colors_map.get(color_id).copied().unwrap_or_else(|| {
        eprintln!("Missing aftik color data for color {color_id:?}!");
        aftik_colors_map.insert(color_id.clone(), color::DEFAULT_COLOR);
        color::DEFAULT_COLOR
    })
}

pub fn get_render_objects_for_entity_with_color(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    aftik_color: AftikColorData,
    properties: &RenderProperties,
    context: &three_d::Context,
) -> Vec<impl three_d::Object> {
    model
        .layers
        .iter()
        .flat_map(|layer| get_render_object_for_layer(layer, pos, properties, aftik_color, context))
        .collect()
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
    let left_x = pos.x - width / 2.;
    let rectangle = three_d::Rectangle::new(
        context,
        three_d::vec2(
            left_x.floor() + width / 2.,
            pos.y + height / 2. - f32::from(layer.positioning.y_offset),
        ),
        three_d::degrees(0.),
        width * direction,
        height,
    );

    let color = layer.color.get_color(&aftik_color);
    let material = texture_color_material(
        &layer.texture,
        three_d::vec4(
            f32::from(color.r) / 255.,
            f32::from(color.g) / 255.,
            f32::from(color.b) / 255.,
            1.,
        ),
    );

    Some(three_d::Gm::new(rectangle, material))
}

pub fn rect(
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

pub fn default_render_camera(viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_2d(viewport);
    render_camera.disable_tone_and_color_mapping();
    render_camera
}

pub fn get_render_camera(camera: &crate::Camera, viewport: three_d::Viewport) -> three_d::Camera {
    let mut render_camera = three_d::Camera::new_orthographic(
        viewport,
        three_d::vec3(
            camera.camera_x + viewport.width as f32 * 0.5,
            viewport.height as f32 * 0.5,
            1.0,
        ),
        three_d::vec3(
            camera.camera_x + viewport.width as f32 * 0.5,
            viewport.height as f32 * 0.5,
            0.0,
        ),
        three_d::vec3(0.0, 1.0, 0.0),
        viewport.height as f32,
        0.0,
        10.0,
    );
    render_camera.disable_tone_and_color_mapping();
    render_camera
}

pub fn color_material(color: three_d::Vec4) -> impl three_d::Material {
    UnalteredColorMaterial(
        three_d::ColorMaterial {
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    )
}

fn texture_material(texture: &three_d::Texture2DRef) -> impl three_d::Material + Clone {
    three_d::ColorMaterial {
        texture: Some(texture.clone()),
        render_states: three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            blend: three_d::Blend::STANDARD_TRANSPARENCY,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn texture_color_material(
    texture: &three_d::Texture2DRef,
    color: three_d::Vec4,
) -> impl three_d::Material {
    UnalteredColorMaterial(
        three_d::ColorMaterial {
            texture: Some(texture.clone()),
            render_states: three_d::RenderStates {
                write_mask: three_d::WriteMask::COLOR,
                blend: three_d::Blend::STANDARD_TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        },
        color,
    )
}

pub struct UnalteredColorMaterial(pub three_d::ColorMaterial, pub three_d::Vec4);

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
        program.use_uniform("surfaceColor", self.1);
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

pub fn draw_in_order(
    objects: &[impl three_d::Object],
    camera: &three_d::Camera,
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
