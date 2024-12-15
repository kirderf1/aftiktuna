pub use self::model::LazilyLoadedModels;
use aftiktuna::asset::background::BGData;
use aftiktuna::asset::color::{self, AftikColorData};
use aftiktuna::asset::{self, TextureLoader};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId};
use aftiktuna::view::area::RenderProperties;
use macroquad::math::Vec2;
use macroquad::prelude::ImageFormat;
use macroquad::texture::Texture2D;
use serde_json::Error as JsonError;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Read;

pub mod background;
pub mod model;

pub struct RenderAssets {
    backgrounds: HashMap<BackgroundId, BGData<Texture2D>>,
    pub models: LazilyLoadedModels,
    aftik_colors: HashMap<AftikColorId, AftikColorData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
}

impl RenderAssets {
    pub fn lookup_background(&self, texture_id: &BackgroundId) -> &BGData<Texture2D> {
        self.backgrounds
            .get(texture_id)
            .unwrap_or_else(|| self.backgrounds.get(&BackgroundId::blank()).unwrap())
    }
}

fn lookup_or_log_aftik_color(
    aftik_color: &AftikColorId,
    aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
) -> AftikColorData {
    aftik_colors_map
        .get(aftik_color)
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing aftik color data for color {aftik_color:?}!");
            aftik_colors_map.insert(aftik_color.clone(), color::DEFAULT_COLOR);
            color::DEFAULT_COLOR
        })
}

pub fn draw_object(
    model_id: &ModelId,
    properties: &RenderProperties,
    use_wield_offset: bool,
    pos: Vec2,
    assets: &mut RenderAssets,
) {
    let model = assets.models.lookup_model(model_id);
    let aftik_color_data = properties
        .aftik_color
        .as_ref()
        .map_or(color::DEFAULT_COLOR, |aftik_color| {
            lookup_or_log_aftik_color(aftik_color, &mut assets.aftik_colors)
        });

    model.draw(pos, use_wield_offset, properties, &aftik_color_data);
}

pub fn load_texture(name: impl AsRef<str>) -> Result<Texture2D, io::Error> {
    let path = format!("assets/texture/{}.png", name.as_ref());

    let mut bytes = vec![];
    File::open(path)?.read_to_end(&mut bytes)?;
    Ok(Texture2D::from_file_with_format(
        &bytes,
        Some(ImageFormat::Png),
    ))
}

struct InPlaceLoader;

impl TextureLoader<Texture2D, io::Error> for InPlaceLoader {
    fn load_texture(&mut self, name: String) -> Result<Texture2D, io::Error> {
        load_texture(name)
    }
}

#[derive(Default)]
pub struct CachedTextures(HashMap<String, Texture2D>);

impl TextureLoader<Texture2D, io::Error> for CachedTextures {
    fn load_texture(&mut self, name: String) -> Result<Texture2D, std::io::Error> {
        if let Some(texture) = self.0.get(&name) {
            return Ok(texture.clone());
        }

        let texture = load_texture(&name)?;
        self.0.insert(name, texture.clone());
        Ok(texture)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Json(JsonError),
    MissingBlankBackground,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Error::Json(value)
    }
}

impl From<asset::Error> for Error {
    fn from(value: asset::Error) -> Self {
        match value {
            asset::Error::IO(error) => Self::IO(error),
            asset::Error::Json(error) => Self::Json(error),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(error) => Display::fmt(error, f),
            Error::Json(error) => Display::fmt(error, f),
            Error::MissingBlankBackground => {
                Display::fmt("Missing Background: Blank background texture must exist", f)
            }
        }
    }
}

pub fn load_assets() -> Result<RenderAssets, Error> {
    Ok(RenderAssets {
        backgrounds: background::load_backgrounds()?,
        models: model::prepare()?,
        aftik_colors: color::load_aftik_color_data()?,
        left_mouse_icon: load_texture("left_mouse")?,
        side_arrow: load_texture("side_arrow")?,
    })
}
