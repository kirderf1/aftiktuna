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

use crate::core::{area::BackgroundId, position::Coord};

pub struct BGData {
    pub texture: BGTexture,
    pub portrait: BGPortrait,
}

pub enum BGTexture {
    Fixed(Parallax<Texture2D>),
    Repeating(Parallax<Texture2D>),
}

impl BGTexture {
    pub fn draw(&self, offset: Coord, camera_space: Rect) {
        let offset = offset as f32 * 120.;
        match self {
            BGTexture::Fixed(parallax) => {
                for layer in &parallax.layers {
                    let layer_x = -60. * layer.move_factor
                        + camera_space.x * (1. - layer.move_factor)
                        - offset;
                    texture::draw_texture(layer.texture, layer_x, 0., color::WHITE)
                }
            }
            BGTexture::Repeating(parallax) => {
                for layer in &parallax.layers {
                    let layer_x = camera_space.x * (1. - layer.move_factor) - offset;
                    let texture = layer.texture;
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
                }
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
    fn load(self) -> Result<BGData, io::Error> {
        Ok(BGData {
            texture: self.texture.load()?,
            portrait: self.portrait.load()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawBGTexture {
    Centered { texture: String },
    Fixed(RawParallax),
    Repeating(RawParallax),
}

impl RawBGTexture {
    fn load(&self) -> Result<BGTexture, io::Error> {
        Ok(match self {
            RawBGTexture::Centered { texture } => BGTexture::Fixed(Parallax {
                layers: vec![ParallaxLayer {
                    texture: load_texture(texture)?,
                    move_factor: 0.,
                }],
            }),
            RawBGTexture::Fixed(raw_parallax) => BGTexture::Fixed(raw_parallax.load()?),
            RawBGTexture::Repeating(raw_parallax) => BGTexture::Repeating(raw_parallax.load()?),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RawParallax {
    Texture { texture: String },
    Parallax(Parallax<String>),
}

impl RawParallax {
    fn load(&self) -> Result<Parallax<Texture2D>, io::Error> {
        match self {
            RawParallax::Texture { texture } => Ok(Parallax {
                layers: vec![ParallaxLayer {
                    texture: load_texture(texture)?,
                    move_factor: 1.,
                }],
            }),
            RawParallax::Parallax(parallax) => parallax.load(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parallax<T> {
    layers: Vec<ParallaxLayer<T>>,
}

impl Parallax<String> {
    fn load(&self) -> Result<Parallax<Texture2D>, io::Error> {
        Ok(Parallax {
            layers: self
                .layers
                .iter()
                .map(ParallaxLayer::load)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ParallaxLayer<T> {
    texture: T,
    move_factor: f32,
}

impl ParallaxLayer<String> {
    fn load(&self) -> Result<ParallaxLayer<Texture2D>, io::Error> {
        Ok(ParallaxLayer {
            texture: load_texture(&self.texture)?,
            move_factor: self.move_factor,
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
    fn load(self) -> Result<BGPortrait, io::Error> {
        Ok(match self {
            RawBGPortrait::Color(color) => {
                BGPortrait::Color([color[0], color[1], color[2], 255].into())
            }
            RawBGPortrait::Texture(texture) => BGPortrait::Texture(load_texture(&texture)?),
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
    let mut backgrounds = HashMap::new();
    for (bg_type, raw_data) in raw_backgrounds {
        insert_or_log(&mut backgrounds, bg_type, raw_data.load());
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
        .load()
        .unwrap()
}

fn load_texture(texture: &str) -> Result<Texture2D, io::Error> {
    super::load_texture(format!("background/{texture}"))
}
