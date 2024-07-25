use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io;

use indexmap::IndexMap;
use macroquad::color::{self, Color};
use macroquad::texture::{self, Texture2D};
use macroquad::window;
use serde::{Deserialize, Serialize};

use crate::core::area::BackgroundId;
use crate::core::position::Coord;
use crate::macroquad_interface;
use crate::macroquad_interface::camera::HorizontalDraggableCamera;

use super::{CachedTextures, TextureLoader};

pub const DATA_FILE_PATH: &str = "assets/texture/background/backgrounds.json";

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
                macroquad_interface::WINDOW_HEIGHT_F - y - texture.height(),
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
                    (camera.x_start + macroquad_interface::WINDOW_WIDTH_F - layer_x)
                        / texture.width(),
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

#[derive(Serialize, Deserialize)]
pub struct RawBGData {
    #[serde(flatten)]
    pub primary: RawPrimaryBGData,
    #[serde(flatten)]
    portrait: RawPortraitBGData,
}

impl RawBGData {
    pub fn load(&self, loader: &mut impl TextureLoader) -> Result<BGData, io::Error> {
        Ok(BGData {
            primary: self.primary.load(loader)?,
            portrait: self.portrait.load(loader)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ParallaxLayerOrList", into = "ParallaxLayerOrList")]
pub struct RawPrimaryBGData(pub Parallax<String>);

impl RawPrimaryBGData {
    fn load(&self, loader: &mut impl TextureLoader) -> Result<PrimaryBGData, io::Error> {
        Ok(PrimaryBGData(self.0.load(loader)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ParallaxLayerOrList {
    Layer(ParallaxLayer<String>),
    Parallax(Parallax<String>),
}

impl From<RawPrimaryBGData> for ParallaxLayerOrList {
    fn from(RawPrimaryBGData(parallax): RawPrimaryBGData) -> Self {
        if parallax.layers.len() != 1 {
            Self::Parallax(parallax)
        } else {
            Self::Layer(parallax.layers.into_iter().next().unwrap())
        }
    }
}

impl From<ParallaxLayerOrList> for RawPrimaryBGData {
    fn from(value: ParallaxLayerOrList) -> Self {
        Self(match value {
            ParallaxLayerOrList::Layer(layer) => Parallax {
                layers: vec![layer],
            },
            ParallaxLayerOrList::Parallax(parallax) => parallax,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parallax<T> {
    pub layers: Vec<ParallaxLayer<T>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallaxLayer<T> {
    pub texture: T,
    #[serde(default = "default_move_factor")]
    pub move_factor: f32,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub is_looping: bool,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub offset: Offset,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offset {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub x: i16,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub y: i16,
}

#[derive(Serialize, Deserialize)]
enum RawPortraitBGData {
    #[serde(rename = "portrait_color")]
    Color([u8; 3]),
    #[serde(rename = "portrait_texture")]
    Texture(String),
}

impl RawPortraitBGData {
    fn load(&self, loader: &mut impl TextureLoader) -> Result<PortraitBGData, io::Error> {
        Ok(match self {
            RawPortraitBGData::Color(color) => {
                PortraitBGData::Color([color[0], color[1], color[2], 255].into())
            }
            RawPortraitBGData::Texture(texture) => {
                PortraitBGData::Texture(load_texture(texture, loader)?)
            }
        })
    }
}

fn load_raw_backgrounds() -> Result<HashMap<BackgroundId, RawBGData>, super::Error> {
    let file = File::open(DATA_FILE_PATH)?;
    Ok(serde_json::from_reader::<
        _,
        HashMap<BackgroundId, RawBGData>,
    >(file)?)
}

pub fn load_index_map_backgrounds() -> Result<IndexMap<BackgroundId, RawBGData>, super::Error> {
    let file = File::open(DATA_FILE_PATH)?;
    Ok(serde_json::from_reader::<
        _,
        IndexMap<BackgroundId, RawBGData>,
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

pub fn load_background_for_testing() -> PrimaryBGData {
    load_raw_backgrounds()
        .unwrap()
        .get(&BackgroundId::new("forest"))
        .unwrap()
        .primary
        .load(&mut super::InPlaceLoader)
        .unwrap()
}

fn load_texture(texture: &str, loader: &mut impl TextureLoader) -> Result<Texture2D, io::Error> {
    loader.load_texture(format!("background/{texture}"))
}
