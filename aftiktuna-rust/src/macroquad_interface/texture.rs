use crate::core::area::BackgroundId;
use crate::core::position::{Coord, Direction};
use crate::core::{AftikColorId, ModelId};
use crate::view::area::{ObjectRenderData, RenderProperties};
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::file::FileError;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::prelude::ImageFormat;
use egui_macroquad::macroquad::texture::{draw_texture, Texture2D};
use egui_macroquad::macroquad::window;
use serde::Deserialize;
use serde_json::Error as JsonError;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Read;

use self::background::{BGData, BGPortrait, BGTexture};
pub use self::model::LazilyLoadedModels;

mod background;
mod model;

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
    let mut pos = pos;
    if use_wield_offset {
        pos.y += model.wield_offset.y;
        pos.x += match properties.direction {
            Direction::Left => -model.wield_offset.x,
            Direction::Right => model.wield_offset.x,
        }
    }
    let aftik_color_data = properties
        .aftik_color
        .as_ref()
        .map_or(DEFAULT_COLOR, |aftik_color| {
            lookup_or_log_aftik_color(aftik_color, &mut assets.aftik_colors)
        });
    for layer in &model.layers {
        layer.draw(pos, properties, &aftik_color_data);
    }
}

pub fn get_rect_for_object(
    object_data: &ObjectRenderData,
    assets: &mut RenderAssets,
    pos: Vec2,
) -> Rect {
    let model = assets.models.lookup_model(&object_data.texture_type);
    model
        .layers
        .iter()
        .filter(|&layer| layer.is_active(&object_data.properties))
        .fold(Rect::new(pos.x, pos.y, 0., 0.), |rect, layer| {
            rect.combine_with(layer.size(pos))
        })
}

const DEFAULT_COLOR: AftikColorData = AftikColorData {
    primary_color: RGBColor {
        r: 255,
        g: 255,
        b: 255,
    },
    secondary_color: RGBColor { r: 0, g: 0, b: 0 },
};

#[derive(Clone, Deserialize)]
pub struct AftikColorData {
    primary_color: RGBColor,
    secondary_color: RGBColor,
}

#[derive(Clone, Copy, Deserialize)]
struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl From<RGBColor> for Color {
    fn from(value: RGBColor) -> Self {
        Color::from_rgba(value.r, value.g, value.b, 255)
    }
}

pub fn draw_background(
    texture_id: &BackgroundId,
    offset: Coord,
    camera_space: Rect,
    assets: &RenderAssets,
) {
    let offset = offset as f32 * 120.;
    match assets.lookup_background(texture_id).texture {
        BGTexture::Centered(texture) => draw_texture(texture, camera_space.x - offset, 0., WHITE),
        BGTexture::Fixed(texture) => draw_texture(texture, -60. - offset, 0., WHITE),
        BGTexture::Repeating(texture) => {
            let start_x =
                texture.width() * f32::floor((camera_space.x + offset) / texture.width()) - offset;
            draw_texture(texture, start_x, 0., WHITE);
            draw_texture(texture, start_x + texture.width(), 0., WHITE);
        }
    }
}

pub fn draw_background_portrait(background_id: &BackgroundId, assets: &RenderAssets) {
    match assets.lookup_background(background_id).portrait {
        BGPortrait::Color(color) => window::clear_background(color),
        BGPortrait::Texture(texture) => draw_texture(texture, 0., 0., WHITE),
    }
}

fn load_texture(name: impl Borrow<str>) -> Result<Texture2D, io::Error> {
    let path = format!("assets/texture/{}.png", name.borrow());

    let mut bytes = vec![];
    File::open(path)?.read_to_end(&mut bytes)?;
    Ok(Texture2D::from_file_with_format(
        &bytes,
        Some(ImageFormat::Png),
    ))
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

pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorData>, Error> {
    let file = File::open("assets/aftik_colors.json")?;
    Ok(serde_json::from_reader::<
        _,
        HashMap<AftikColorId, AftikColorData>,
    >(file)?)
}
