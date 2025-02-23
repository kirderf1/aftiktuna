use aftiktuna::asset::background::{load_raw_backgrounds, BGData, PortraitBGData};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::position::Coord;
use std::collections::HashMap;

pub struct BackgroundMap(HashMap<BackgroundId, BGData<three_d::Texture2DRef>>);

impl BackgroundMap {
    pub fn load(context: three_d::Context) -> Self {
        let mut texture_loader = super::CachedLoader::new(context);
        let background_data = load_raw_backgrounds().unwrap();
        Self(
            background_data
                .into_iter()
                .map(|(id, data)| (id, data.load(&mut texture_loader).unwrap()))
                .collect(),
        )
    }

    fn get_or_default<'a>(&'a self, id: &BackgroundId) -> &'a BGData<three_d::Texture2DRef> {
        self.0
            .get(id)
            .or_else(|| self.0.get(&BackgroundId::blank()))
            .expect("Missing blank texture")
    }

    pub fn get_render_objects_for_primary(
        &self,
        id: &BackgroundId,
        background_offset: Coord,
        camera_x: f32,
        context: &three_d::Context,
    ) -> Vec<impl three_d::Object> {
        let background = self.get_or_default(id);
        let offset = background_offset as f32 * 120.;

        background
            .primary
            .0
            .layers
            .iter()
            .flat_map(|layer| {
                let width = layer.texture.width() as f32;
                let height = layer.texture.height() as f32;
                let layer_x =
                    f32::from(layer.offset.x) + camera_x * (1. - layer.move_factor) - offset;
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

    pub fn draw_secondary(
        &mut self,
        background_id: &BackgroundId,
        screen: &three_d::RenderTarget<'_>,
        frame_input: &three_d::FrameInput,
    ) {
        let background = self.get_or_default(background_id);
        match &background.portrait {
            &PortraitBGData::Color([r, g, b]) => {
                screen.clear(three_d::ClearState::color(
                    f32::from(r),
                    f32::from(g),
                    f32::from(b),
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
                        ..Default::default()
                    },
                );

                let render_camera = super::default_render_camera(frame_input.viewport);
                screen.render(&render_camera, [background_object], &[]);
            }
        };
    }
}
