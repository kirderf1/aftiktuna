use crate::core::area::BackgroundId;
use crate::core::{AftikColorId, ModelId};
use crate::view::area::RenderProperties;
use egui_macroquad::egui::Color32;
use egui_macroquad::macroquad::color::Color;
use egui_macroquad::macroquad::file::FileError;
use egui_macroquad::macroquad::math::Vec2;
use egui_macroquad::macroquad::prelude::ImageFormat;
use egui_macroquad::macroquad::texture::Texture2D;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Read;

use self::background::BGData;
pub use self::model::LazilyLoadedModels;

pub mod background;
pub mod model;

pub struct RenderAssets {
    backgrounds: HashMap<BackgroundId, BGData>,
    pub models: LazilyLoadedModels,
    aftik_colors: HashMap<AftikColorId, AftikColorData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
}

impl RenderAssets {
    pub fn lookup_background(&self, texture_id: &BackgroundId) -> &BGData {
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
            aftik_colors_map.insert(aftik_color.clone(), DEFAULT_COLOR);
            DEFAULT_COLOR
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
        .map_or(DEFAULT_COLOR, |aftik_color| {
            lookup_or_log_aftik_color(aftik_color, &mut assets.aftik_colors)
        });

    model.draw(pos, use_wield_offset, properties, &aftik_color_data);
}

pub const DEFAULT_COLOR: AftikColorData = AftikColorData {
    primary_color: RGBColor::new(255, 255, 255),
    secondary_color: RGBColor::new(0, 0, 0),
};

#[derive(Clone, Serialize, Deserialize)]
pub struct AftikColorData {
    pub primary_color: RGBColor,
    pub secondary_color: RGBColor,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RGBColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<RGBColor> for Color {
    fn from(RGBColor { r, g, b }: RGBColor) -> Self {
        Color::from_rgba(r, g, b, 255)
    }
}

impl From<RGBColor> for Color32 {
    fn from(RGBColor { r, g, b }: RGBColor) -> Self {
        Color32::from_rgb(r, g, b)
    }
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

pub trait TextureLoader {
    fn load_texture(&mut self, name: String) -> Result<Texture2D, io::Error>;
}

struct InPlaceLoader;

impl TextureLoader for InPlaceLoader {
    fn load_texture(&mut self, name: String) -> Result<Texture2D, io::Error> {
        load_texture(name)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Macroquad(FileError),
    Json(JsonError),
    MissingBlankBackground,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<FileError> for Error {
    fn from(value: FileError) -> Self {
        Error::Macroquad(value)
    }
}

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Error::Json(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(error) => Display::fmt(error, f),
            Error::Macroquad(error) => Display::fmt(error, f),
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
        aftik_colors: load_aftik_color_data()?,
        left_mouse_icon: load_texture("left_mouse")?,
        side_arrow: load_texture("side_arrow")?,
    })
}

pub const AFTIK_COLORS_PATH: &str = "assets/aftik_colors.json";

pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorData>, Error> {
    let file = File::open(AFTIK_COLORS_PATH)?;
    Ok(serde_json::from_reader::<
        _,
        HashMap<AftikColorId, AftikColorData>,
    >(file)?)
}
