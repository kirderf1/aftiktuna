use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io;

use egui_macroquad::macroquad::color::Color;
use egui_macroquad::macroquad::texture::Texture2D;
use serde::{Deserialize, Serialize};

use crate::core::area::BackgroundType;

pub struct BGData {
    pub texture: BGTexture,
    pub portrait: BGPortrait,
}

pub enum BGTexture {
    Centered(Texture2D),
    Fixed(Texture2D),
    Repeating(Texture2D),
}

pub enum BGPortrait {
    Color(Color),
    Texture(Texture2D),
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
    Fixed { texture: String },
    Repeating { texture: String },
}

impl RawBGTexture {
    fn load(self) -> Result<BGTexture, io::Error> {
        Ok(match self {
            RawBGTexture::Centered { texture } => {
                BGTexture::Centered(super::load_texture(format!("background/{texture}"))?)
            }
            RawBGTexture::Fixed { texture } => {
                BGTexture::Fixed(super::load_texture(format!("background/{texture}"))?)
            }
            RawBGTexture::Repeating { texture } => {
                BGTexture::Repeating(super::load_texture(format!("background/{texture}"))?)
            }
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
            RawBGPortrait::Texture(texture) => {
                BGPortrait::Texture(super::load_texture(format!("background/{texture}"))?)
            }
        })
    }
}

pub fn load_backgrounds() -> Result<HashMap<BackgroundType, BGData>, super::Error> {
    let file = File::open("assets/texture/background/backgrounds.json")?;
    let raw_backgrounds: HashMap<BackgroundType, RawBGData> = serde_json::from_reader(file)?;
    let mut backgrounds = HashMap::new();
    for (bg_type, raw_data) in raw_backgrounds {
        insert_or_log(&mut backgrounds, bg_type, raw_data.load());
    }

    backgrounds
        .get(&BackgroundType::blank())
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
