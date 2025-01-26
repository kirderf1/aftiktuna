use aftiktuna::asset::background::{load_raw_backgrounds, BGData, PortraitBGData};
use aftiktuna::core::area::BackgroundId;
use std::collections::HashMap;

pub struct BackgroundMap(HashMap<BackgroundId, BGData<three_d::Texture2DRef>>);

impl BackgroundMap {
    pub fn load(context: three_d::Context) -> Self {
        let mut texture_loader = super::CachedLoader(HashMap::new(), context);
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
        context: &three_d::Context,
    ) -> Vec<impl three_d::Object> {
        let background = self.get_or_default(id);
        background
            .primary
            .0
            .layers
            .iter()
            .map(|layer| {
                three_d::Gm::new(
                    three_d::Rectangle::new(
                        context,
                        three_d::vec2(400., 300.),
                        three_d::degrees(0.),
                        800.,
                        600.,
                    ),
                    three_d::ColorMaterial {
                        texture: Some(layer.texture.clone()),
                        render_states: three_d::RenderStates {
                            depth_test: three_d::DepthTest::Always,
                            blend: three_d::Blend::STANDARD_TRANSPARENCY,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
            })
            .collect()
    }

    pub fn get_render_object_for_secondary(
        &self,
        id: &BackgroundId,
        context: &three_d::Context,
    ) -> impl three_d::Object {
        let background = self.get_or_default(id);
        let material = match &background.portrait {
            &PortraitBGData::Color(color) => three_d::ColorMaterial {
                color: color.into(),
                ..Default::default()
            },
            PortraitBGData::Texture(texture) => three_d::ColorMaterial {
                texture: Some(texture.clone()),
                ..Default::default()
            },
        };
        three_d::Gm::new(
            three_d::Rectangle::new(
                context,
                three_d::vec2(400., 300.),
                three_d::degrees(0.),
                800.,
                600.,
            ),
            material,
        )
    }
}
