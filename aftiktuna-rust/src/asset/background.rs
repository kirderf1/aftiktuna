use super::TextureLoader;
use crate::core::area::BackgroundId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct BGData<T> {
    #[serde(flatten)]
    pub primary: PrimaryBGData<T>,
    #[serde(flatten)]
    pub portrait: PortraitBGData<T>,
}

impl BGData<String> {
    pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<BGData<T>, E> {
        Ok(BGData {
            primary: self.primary.load(loader)?,
            portrait: self.portrait.load(loader)?,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ParallaxLayerOrList<T>")]
pub struct PrimaryBGData<T>(pub Parallax<T>);

impl<T: Serialize> Serialize for PrimaryBGData<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let [layer] = &self.0.layers[..] {
            layer.serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl PrimaryBGData<String> {
    pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<PrimaryBGData<T>, E> {
        Ok(PrimaryBGData(self.0.load(loader)?))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PortraitBGData<T> {
    #[serde(rename = "portrait_color")]
    Color([u8; 3]),
    #[serde(rename = "portrait_texture")]
    Texture(T),
}

impl PortraitBGData<String> {
    pub fn load<T, E>(
        &self,
        loader: &mut impl TextureLoader<T, E>,
    ) -> Result<PortraitBGData<T>, E> {
        Ok(match self {
            PortraitBGData::Color(color) => PortraitBGData::Color(*color),
            PortraitBGData::Texture(texture) => {
                PortraitBGData::Texture(load_texture(texture, loader)?)
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ParallaxLayerOrList<T> {
    Layer(ParallaxLayer<T>),
    Parallax(Parallax<T>),
}

impl<T> From<PrimaryBGData<T>> for ParallaxLayerOrList<T> {
    fn from(PrimaryBGData(parallax): PrimaryBGData<T>) -> Self {
        if parallax.layers.len() != 1 {
            Self::Parallax(parallax)
        } else {
            Self::Layer(parallax.layers.into_iter().next().unwrap())
        }
    }
}

impl<T> From<ParallaxLayerOrList<T>> for PrimaryBGData<T> {
    fn from(value: ParallaxLayerOrList<T>) -> Self {
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
    pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<Parallax<T>, E> {
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
    pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<ParallaxLayer<T>, E> {
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

pub const DATA_FILE_PATH: &str = "assets/texture/background/backgrounds.json";

pub fn load_raw_backgrounds() -> Result<HashMap<BackgroundId, BGData<String>>, super::Error> {
    super::load_from_json(DATA_FILE_PATH)
}

pub fn load_index_map_backgrounds() -> Result<IndexMap<BackgroundId, BGData<String>>, super::Error>
{
    super::load_from_json(DATA_FILE_PATH)
}

fn load_texture<T, E>(texture: &str, loader: &mut impl TextureLoader<T, E>) -> Result<T, E> {
    loader.load_texture(format!("background/{texture}"))
}
