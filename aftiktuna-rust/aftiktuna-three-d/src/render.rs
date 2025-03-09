use aftiktuna::asset;
use aftiktuna::asset::color::{AftikColorData, RGBColor};
use aftiktuna::asset::model::{Model, TextureLayer};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::AftikColorId;
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
    assets: &mut super::Assets,
) {
    match frame {
        Frame::Introduction | Frame::LocationChoice(_) | Frame::Error(_) => {
            let background_objects = assets.backgrounds.get_render_objects_for_primary(
                &BackgroundId::location_choice(),
                0,
                0.,
                &frame_input.context,
            );

            let render_camera = super::default_render_camera(frame_input.viewport);
            screen
                .write::<three_d::RendererError>(|| {
                    for object in background_objects {
                        object.render(&render_camera, &[]);
                    }
                    Ok(())
                })
                .unwrap();
        }
        Frame::AreaView { render_data, .. } => {
            draw_area_view(render_data, camera, screen, frame_input, assets);
        }
        Frame::Dialogue { data, .. } => {
            assets
                .backgrounds
                .draw_secondary(&data.background, screen, frame_input);
        }
        Frame::StoreView { view, .. } => {
            assets
                .backgrounds
                .draw_secondary(&view.background, screen, frame_input);
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
    assets: &mut super::Assets,
) {
    let background_objects = assets.backgrounds.get_render_objects_for_primary(
        &render_data.background,
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

pub fn position_objects<'a>(
    objects: &'a Vec<ObjectRenderData>,
    models: &mut super::LazilyLoadedModels,
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
        coord_to_center_x(coord) - count as f32 * 15.,
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

// Coordinates are mapped like this so that when the left edge of the window is 0,
// coord 3 will be placed in the middle of the window.
fn coord_to_center_x(coord: Coord) -> f32 {
    40. + 120. * coord as f32
}
