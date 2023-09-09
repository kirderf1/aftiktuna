use crate::core::area::BackgroundType;
use crate::core::position::{Coord, Direction};
use crate::view::{AftikColor, ObjectRenderData, TextureType};
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::file::FileError;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::prelude::ImageFormat;
use egui_macroquad::macroquad::texture::{
    draw_texture, draw_texture_ex, DrawTextureParams, Texture2D,
};
use egui_macroquad::macroquad::window;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::io::Read;

pub struct TextureStorage {
    backgrounds: HashMap<BackgroundType, BGData>,
    objects: HashMap<TextureType, TextureData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
    pub portrait: TextureData,
}

impl TextureStorage {
    pub fn lookup_background(&self, texture_type: BackgroundType) -> &BGData {
        self.backgrounds
            .get(&texture_type)
            .unwrap_or_else(|| self.backgrounds.get(&BackgroundType::Blank).unwrap())
    }

    pub fn lookup_texture(&mut self, texture_type: TextureType) -> &TextureData {
        if !self.objects.contains_key(&texture_type) {
            objects::load_or_default(&mut self.objects, texture_type);
        }
        self.objects.get(&texture_type).unwrap()
    }
}

#[derive(Clone)]
pub struct TextureData {
    layers: Vec<TextureLayer>,
    wield_offset: Vec2,
    directional: bool,
    is_mounted: bool,
}

impl TextureData {
    pub fn is_displacing(&self) -> bool {
        !self.is_mounted
    }
}

#[derive(Clone)]
struct TextureLayer {
    texture: Texture2D,
    color: ColorSource,
    dest_size: Vec2,
    y_offset: f32,
    if_cut: Option<bool>,
}

impl TextureLayer {
    fn draw(&self, pos: Vec2, flip_x: bool, aftik_color: Option<AftikColor>, is_cut: bool) {
        if self.if_cut.is_some() && self.if_cut != Some(is_cut) {
            return;
        }

        let x = pos.x - self.dest_size.x / 2.;
        let y = pos.y + self.y_offset - self.dest_size.y;
        draw_texture_ex(
            self.texture,
            x,
            y,
            self.color.get_color(aftik_color),
            DrawTextureParams {
                dest_size: Some(self.dest_size),
                flip_x,
                ..Default::default()
            },
        );
    }

    fn size(&self, pos: Vec2) -> Rect {
        Rect::new(
            pos.x - self.dest_size.x / 2.,
            pos.y - self.dest_size.y + self.y_offset,
            self.dest_size.x,
            self.dest_size.y,
        )
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ColorSource {
    #[default]
    Uncolored,
    Primary,
    Secondary,
}

impl ColorSource {
    fn get_color(self, aftik_color: Option<AftikColor>) -> Color {
        match self {
            ColorSource::Uncolored => WHITE,
            ColorSource::Primary => convert_to_color(aftik_color.unwrap_or_default()).0,
            ColorSource::Secondary => convert_to_color(aftik_color.unwrap_or_default()).1,
        }
    }
}

pub fn draw_object(
    data: &TextureData,
    direction: Direction,
    aftik_color: Option<AftikColor>,
    is_cut: bool,
    use_wield_offset: bool,
    pos: Vec2,
) {
    let mut pos = pos;
    if use_wield_offset {
        pos.y += data.wield_offset.y;
        pos.x += match direction {
            Direction::Left => -data.wield_offset.x,
            Direction::Right => data.wield_offset.x,
        }
    }
    for layer in &data.layers {
        layer.draw(
            pos,
            data.directional && direction == Direction::Left,
            aftik_color,
            is_cut,
        );
    }
}

pub fn get_rect_for_object(
    data: &ObjectRenderData,
    textures: &mut TextureStorage,
    pos: Vec2,
) -> Rect {
    let data = textures.lookup_texture(data.texture_type);
    data.layers[0].size(pos)
}

fn convert_to_color(color: AftikColor) -> (Color, Color) {
    match color {
        AftikColor::Mint => (
            Color::from_rgba(148, 216, 0, 255),
            Color::from_rgba(255, 238, 153, 255),
        ),
        AftikColor::Cerulean => (
            Color::from_rgba(84, 141, 197, 255),
            Color::from_rgba(153, 223, 255, 255),
        ),
        AftikColor::Plum => (
            Color::from_rgba(183, 98, 168, 255),
            Color::from_rgba(255, 177, 132, 255),
        ),
        AftikColor::Green => (
            Color::from_rgba(78, 218, 67, 255),
            Color::from_rgba(192, 232, 255, 255),
        ),
    }
}

pub struct BGData {
    texture: BGTexture,
    portrait: BGPortrait,
}

enum BGTexture {
    Centered(Texture2D),
    Fixed(Texture2D),
    Repeating(Texture2D),
}

enum BGPortrait {
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
                BGTexture::Centered(load_texture(format!("background/{texture}"))?)
            }
            RawBGTexture::Fixed { texture } => {
                BGTexture::Fixed(load_texture(format!("background/{texture}"))?)
            }
            RawBGTexture::Repeating { texture } => {
                BGTexture::Repeating(load_texture(format!("background/{texture}"))?)
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
                BGPortrait::Texture(load_texture(format!("background/{texture}"))?)
            }
        })
    }
}

pub fn draw_background(
    texture_type: BackgroundType,
    offset: Coord,
    camera_space: Rect,
    textures: &TextureStorage,
) {
    let offset = offset as f32 * 120.;
    match textures.lookup_background(texture_type).texture {
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

pub fn draw_background_portrait(background_data: &BGData) {
    match background_data.portrait {
        BGPortrait::Color(color) => window::clear_background(color),
        BGPortrait::Texture(texture) => draw_texture(texture, 0., 0., WHITE),
    }
}

#[derive(Serialize, Deserialize)]
struct RawTextureData {
    layers: Vec<RawTextureLayer>,
    #[serde(default)]
    wield_offset: (f32, f32),
    #[serde(default)]
    fixed: bool,
    #[serde(default)]
    mounted: bool,
}

impl RawTextureData {
    fn load(self) -> Result<TextureData, io::Error> {
        let mut layers = Vec::new();
        for layer in self.layers {
            layers.push(layer.load()?);
        }
        layers.reverse();
        Ok(TextureData {
            layers,
            wield_offset: Vec2::from(self.wield_offset),
            directional: !self.fixed,
            is_mounted: self.mounted,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct RawTextureLayer {
    texture: String,
    #[serde(default)]
    color: ColorSource,
    #[serde(default)]
    size: Option<(f32, f32)>,
    #[serde(default)]
    y_offset: f32,
    #[serde(default)]
    if_cut: Option<bool>,
}

impl RawTextureLayer {
    fn load(self) -> Result<TextureLayer, io::Error> {
        let texture = load_texture(self.texture)?;
        Ok(TextureLayer {
            texture,
            color: self.color,
            dest_size: Vec2::from(
                self.size
                    .unwrap_or_else(|| (texture.width(), texture.height())),
            ),
            y_offset: self.y_offset,
            if_cut: self.if_cut,
        })
    }
}

fn load_texture_data(path: &str) -> Result<TextureData, Error> {
    let file = File::open(format!("assets/texture/{path}.json"))?;
    let data = serde_json::from_reader::<_, RawTextureData>(file)?;
    let data = data.load()?;
    Ok(data)
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
        }
    }
}

pub fn load_textures() -> Result<TextureStorage, Error> {
    Ok(TextureStorage {
        backgrounds: load_backgrounds()?,
        objects: objects::prepare()?,
        left_mouse_icon: load_texture("left_mouse")?,
        side_arrow: load_texture("side_arrow")?,
        portrait: load_texture_data("portrait")?,
    })
}

fn load_backgrounds() -> Result<HashMap<BackgroundType, BGData>, Error> {
    let file = File::open("assets/texture/background/backgrounds.json")?;
    let raw_backgrounds: HashMap<BackgroundType, RawBGData> = serde_json::from_reader(file)?;
    let mut backgrounds = HashMap::new();
    for (bg_type, raw_data) in raw_backgrounds {
        insert_or_log(&mut backgrounds, bg_type, raw_data.load());
    }

    backgrounds
        .get(&BackgroundType::Blank)
        .expect("Blank background texture must exist");

    Ok(backgrounds)
}

mod objects {
    use super::{load_texture_data, Error, TextureData};
    use crate::view::TextureType;
    use std::collections::HashMap;

    pub fn prepare() -> Result<HashMap<TextureType, TextureData>, Error> {
        let mut objects = HashMap::new();

        load(&mut objects, TextureType::Unknown)?;
        load(&mut objects, TextureType::SmallUnknown)?;

        Ok(objects)
    }

    fn load(
        objects: &mut HashMap<TextureType, TextureData>,
        key: impl Into<TextureType>,
    ) -> Result<(), Error> {
        let key = key.into();
        objects.insert(key, load_texture_data(key.path())?);
        Ok(())
    }

    pub fn load_or_default(
        objects: &mut HashMap<TextureType, TextureData>,
        texture_type: TextureType,
    ) {
        let path = texture_type.path();
        let texture_data = load_texture_data(path).unwrap_or_else(|error| {
            eprintln!("Unable to load texture data \"{path}\": {error}");
            if matches!(texture_type, TextureType::Item(_)) {
                objects.get(&TextureType::SmallUnknown).unwrap().clone()
            } else {
                objects.get(&TextureType::Unknown).unwrap().clone()
            }
        });
        objects.insert(texture_type, texture_data);
    }
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
