use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io;

use egui_macroquad::macroquad::color::{self, Color};
use egui_macroquad::macroquad::math::Rect;
use egui_macroquad::macroquad::texture::{self, Texture2D};
use egui_macroquad::macroquad::window;
use serde::{Deserialize, Serialize};

use crate::core::area::BackgroundId;
use crate::core::position::Coord;

use super::{CachedTextures, TextureLoader};

pub struct BGData {
    pub texture: BGTexture,
    pub portrait: BGPortrait,
}

pub struct BGTexture(Parallax<Texture2D>);

impl BGTexture {
    pub fn draw(&self, offset: Coord, camera_space: Rect) {
        let offset = offset as f32 * 120.;
        for layer in &self.0.layers {
            let layer_x = camera_space.x * (1. - layer.move_factor) - offset - layer.offset;
            let texture = layer.texture;
            if layer.is_looping {
                let repeat_count = f32::floor((camera_space.x - layer_x) / texture.width());
                texture::draw_texture(
                    texture,
                    layer_x + texture.width() * repeat_count,
                    0.,
                    color::WHITE,
                );
                texture::draw_texture(
                    texture,
                    layer_x + texture.width() * (repeat_count + 1.),
                    0.,
                    color::WHITE,
                );
            } else {
                texture::draw_texture(texture, layer_x, 0., color::WHITE)
            }
        }
    }
}

pub enum BGPortrait {
    Color(Color),
    Texture(Texture2D),
}

impl BGPortrait {
    pub fn draw(&self) {
        match *self {
            BGPortrait::Color(color) => window::clear_background(color),
            BGPortrait::Texture(texture) => texture::draw_texture(texture, 0., 0., color::WHITE),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct RawBGData {
    #[serde(flatten)]
    texture: RawBGTexture,
    #[serde(flatten)]
    portrait: RawBGPortrait,
}

impl RawBGData {
    fn load(self, loader: &mut impl TextureLoader) -> Result<BGData, io::Error> {
        Ok(BGData {
            texture: self.texture.load(loader)?,
            portrait: self.portrait.load(loader)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RawBGTexture {
    Layer(ParallaxLayer<String>),
    Parallax(Parallax<String>),
}

impl RawBGTexture {
    fn load(&self, loader: &mut impl TextureLoader) -> Result<BGTexture, io::Error> {
        Ok(BGTexture(match self {
            Self::Layer(layer) => Parallax {
                layers: vec![layer.load(loader)?],
            },
            Self::Parallax(parallax) => parallax.load(loader)?,
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parallax<T> {
    layers: Vec<ParallaxLayer<T>>,
}

impl Parallax<String> {
    fn load(&self, loader: &mut impl TextureLoader) -> Result<Parallax<Texture2D>, io::Error> {
        Ok(Parallax {
            layers: self
                .layers
                .iter()
                .map(|layer| layer.load(loader))
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ParallaxLayer<T> {
    texture: T,
    #[serde(default = "default_move_factor")]
    move_factor: f32,
    #[serde(default)]
    is_looping: bool,
    #[serde(default)]
    offset: f32,
}

fn default_move_factor() -> f32 {
    1.
}

impl ParallaxLayer<String> {
    fn load(&self, loader: &mut impl TextureLoader) -> Result<ParallaxLayer<Texture2D>, io::Error> {
        Ok(ParallaxLayer {
            texture: load_texture(&self.texture, loader)?,
            move_factor: self.move_factor,
            is_looping: self.is_looping,
            offset: self.offset,
        })
    }
}

#[derive(Serialize, Deserialize)]
enum RawBGPortrait {
    #[serde(rename = "portrait_color")]
    Color([u8; 3]),
    #[serde(rename = "portrait_texture")]
    Texture(String),
}

impl RawBGPortrait {
    fn load(self, loader: &mut impl TextureLoader) -> Result<BGPortrait, io::Error> {
        Ok(match self {
            RawBGPortrait::Color(color) => {
                BGPortrait::Color([color[0], color[1], color[2], 255].into())
            }
            RawBGPortrait::Texture(texture) => BGPortrait::Texture(load_texture(&texture, loader)?),
        })
    }
}

fn load_raw_backgrounds() -> Result<HashMap<BackgroundId, RawBGData>, super::Error> {
    let file = File::open("assets/texture/background/backgrounds.json")?;
    Ok(serde_json::from_reader::<
        _,
        HashMap<BackgroundId, RawBGData>,
    >(file)?)
}

pub fn load_backgrounds() -> Result<HashMap<BackgroundId, BGData>, super::Error> {
    let raw_backgrounds = load_raw_backgrounds()?;
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

pub fn load_background_for_testing() -> BGTexture {
    load_raw_backgrounds()
        .unwrap()
        .get(&BackgroundId::new("forest"))
        .unwrap()
        .texture
        .load(&mut super::InPlaceLoader)
        .unwrap()
}

fn load_texture(texture: &str, loader: &mut impl TextureLoader) -> Result<Texture2D, io::Error> {
    loader.load_texture(format!("background/{texture}"))
}
