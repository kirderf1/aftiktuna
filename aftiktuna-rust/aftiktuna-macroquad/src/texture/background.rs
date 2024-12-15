use super::CachedTextures;
use crate::camera::HorizontalDraggableCamera;
use aftiktuna::asset::background::{
    self, Parallax, RawBGData, RawPortraitBGData, RawPrimaryBGData,
};
use aftiktuna::asset::TextureLoader;
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::position::Coord;
use macroquad::color::{self, Color};
use macroquad::texture::{self, Texture2D};
use macroquad::window;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

pub struct BGData {
    pub primary: PrimaryBGData,
    pub portrait: PortraitBGData,
}

pub struct PrimaryBGData(Parallax<Texture2D>);

impl PrimaryBGData {
    pub fn draw(&self, offset: Coord, camera: &HorizontalDraggableCamera) {
        fn draw_background_texture(texture: &Texture2D, x: f32, y: f32) {
            texture::draw_texture(
                texture,
                x,
                crate::WINDOW_HEIGHT_F - y - texture.height(),
                color::WHITE,
            );
        }

        let offset = offset as f32 * 120.;
        for layer in &self.0.layers {
            let layer_x =
                f32::from(layer.offset.x) + camera.x_start * (1. - layer.move_factor) - offset;
            let layer_y = f32::from(layer.offset.y);
            let texture = &layer.texture;

            if layer.is_looping {
                let repeat_start = f32::floor((camera.x_start - layer_x) / texture.width()) as i16;
                let repeat_end = f32::floor(
                    (camera.x_start + crate::WINDOW_WIDTH_F - layer_x) / texture.width(),
                ) as i16;
                for repeat_index in repeat_start..=repeat_end {
                    draw_background_texture(
                        texture,
                        layer_x + texture.width() * f32::from(repeat_index),
                        layer_y,
                    );
                }
            } else {
                draw_background_texture(texture, layer_x, layer_y);
            }
        }
    }
}

pub enum PortraitBGData {
    Color(Color),
    Texture(Texture2D),
}

impl PortraitBGData {
    pub fn draw(&self) {
        match self {
            PortraitBGData::Color(color) => window::clear_background(*color),
            PortraitBGData::Texture(texture) => {
                texture::draw_texture(texture, 0., 0., color::WHITE);
            }
        }
    }
}

pub fn load_bg_data<E>(
    bg_data: &RawBGData,
    loader: &mut impl TextureLoader<Texture2D, E>,
) -> Result<BGData, E> {
    Ok(BGData {
        primary: load_primary(&bg_data.primary, loader)?,
        portrait: load_portrait(&bg_data.portrait, loader)?,
    })
}

fn load_primary<E>(
    primary_data: &RawPrimaryBGData,
    loader: &mut impl TextureLoader<Texture2D, E>,
) -> Result<PrimaryBGData, E> {
    Ok(PrimaryBGData(primary_data.0.load(loader)?))
}

fn load_portrait<E>(
    portait_data: &RawPortraitBGData,
    loader: &mut impl TextureLoader<Texture2D, E>,
) -> Result<PortraitBGData, E> {
    Ok(match portait_data {
        RawPortraitBGData::Color(color) => {
            PortraitBGData::Color([color[0], color[1], color[2], 255].into())
        }
        RawPortraitBGData::Texture(texture) => {
            PortraitBGData::Texture(background::load_texture(texture, loader)?)
        }
    })
}

pub fn load_backgrounds() -> Result<HashMap<BackgroundId, BGData>, super::Error> {
    let raw_backgrounds = background::load_raw_backgrounds()?;
    let mut textures = CachedTextures::default();
    let mut backgrounds = HashMap::new();
    for (bg_type, raw_data) in raw_backgrounds {
        insert_or_log(
            &mut backgrounds,
            bg_type,
            load_bg_data(&raw_data, &mut textures),
        );
    }

    backgrounds
        .get(&BackgroundId::blank())
        .ok_or(super::Error::MissingBlankBackground)?;

    Ok(backgrounds)
}

fn insert_or_log<K: Eq + Hash, V, D: Display>(
    objects: &mut HashMap<K, V>,
    key: K,
    result: Result<V, D>,
) {
    match result {
        Ok(value) => {
            objects.insert(key, value);
        }
        Err(error) => {
            eprintln!("Unable to load texture: {error}");
        }
    }
}

pub fn load_background_for_testing() -> PrimaryBGData {
    load_primary(
        &background::load_raw_backgrounds()
            .unwrap()
            .get(&BackgroundId::new("forest"))
            .unwrap()
            .primary,
        &mut super::InPlaceLoader,
    )
    .unwrap()
}
