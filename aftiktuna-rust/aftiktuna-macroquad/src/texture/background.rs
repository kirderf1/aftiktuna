use super::CachedTextures;
use crate::camera::HorizontalDraggableCamera;
use aftiktuna::asset::background::{self, BGData, PortraitBGData, PrimaryBGData};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::position::Coord;
use macroquad::color::{self, Color};
use macroquad::texture::{self, Texture2D};
use macroquad::window;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

pub fn draw_primary(
    primary_data: &PrimaryBGData<Texture2D>,
    offset: Coord,
    camera: &HorizontalDraggableCamera,
) {
    fn draw_background_texture(texture: &Texture2D, x: f32, y: f32) {
        texture::draw_texture(
            texture,
            x,
            crate::WINDOW_HEIGHT_F - y - texture.height(),
            color::WHITE,
        );
    }

    let offset = offset as f32 * 120.;
    for layer in &primary_data.0.layers {
        let layer_x =
            f32::from(layer.offset.x) + camera.x_start * (1. - layer.move_factor) - offset;
        let layer_y = f32::from(layer.offset.y);
        let texture = &layer.texture;

        if layer.is_looping {
            let repeat_start = f32::floor((camera.x_start - layer_x) / texture.width()) as i16;
            let repeat_end =
                f32::floor((camera.x_start + crate::WINDOW_WIDTH_F - layer_x) / texture.width())
                    as i16;
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

pub fn draw_portrait(portait_data: &PortraitBGData<Texture2D>) {
    match portait_data {
        &PortraitBGData::Color([r, g, b]) => {
            window::clear_background(Color::from_rgba(r, g, b, 255))
        }
        PortraitBGData::Texture(texture) => {
            texture::draw_texture(texture, 0., 0., color::WHITE);
        }
    }
}

pub fn load_backgrounds() -> Result<HashMap<BackgroundId, BGData<Texture2D>>, super::Error> {
    let raw_backgrounds = background::load_raw_backgrounds()?;
    let mut textures = CachedTextures::default();
    let mut backgrounds = HashMap::new();
    for (bg_type, raw_data) in raw_backgrounds {
        insert_or_log(&mut backgrounds, bg_type, raw_data.load(&mut textures));
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

pub fn load_background_for_testing() -> PrimaryBGData<Texture2D> {
    background::load_raw_backgrounds()
        .unwrap()
        .get(&BackgroundId::new("forest"))
        .unwrap()
        .primary
        .load(&mut super::InPlaceLoader)
        .unwrap()
}
