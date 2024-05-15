use crate::core::area::BackgroundType;
use crate::core::position::{Coord, Direction};
use crate::view::area::RenderProperties;
use crate::view::area::{AftikColorId, ObjectRenderData, TextureType};
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

pub struct RenderAssets {
    backgrounds: HashMap<BackgroundType, BGData>,
    pub object_textures: LazilyLoadedObjectTextures,
    pub aftik_colors: HashMap<AftikColorId, AftikColorData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
    pub portrait: TextureData,
}

impl RenderAssets {
    pub fn lookup_background(&self, texture_type: &BackgroundType) -> &BGData {
        self.backgrounds
            .get(texture_type)
            .unwrap_or_else(|| self.backgrounds.get(&BackgroundType::blank()).unwrap())
    }
}

pub struct LazilyLoadedObjectTextures {
    loaded_textures: HashMap<TextureType, TextureData>,
}

impl LazilyLoadedObjectTextures {
    pub fn lookup_texture(&mut self, texture_type: &TextureType) -> &TextureData {
        if !self.loaded_textures.contains_key(texture_type) {
            objects::load_or_default(&mut self.loaded_textures, texture_type);
        }
        self.loaded_textures.get(texture_type).unwrap()
    }
}

#[derive(Clone)]
pub struct TextureData {
    layers: Vec<TextureLayer>,
    wield_offset: Vec2,
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
    directional: bool,
    if_cut: Option<bool>,
    if_alive: Option<bool>,
}

impl TextureLayer {
    fn draw(
        &self,
        pos: Vec2,
        properties: &RenderProperties,
        aftik_colors_map: &HashMap<AftikColorId, AftikColorData>,
    ) {
        if !self.is_active(properties) {
            return;
        }

        let x = pos.x - self.dest_size.x / 2.;
        let y = pos.y + self.y_offset - self.dest_size.y;
        draw_texture_ex(
            self.texture,
            x,
            y,
            self.color
                .get_color(properties.aftik_color.clone(), aftik_colors_map),
            DrawTextureParams {
                dest_size: Some(self.dest_size),
                flip_x: self.directional && properties.direction == Direction::Left,
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

    fn is_active(&self, properties: &RenderProperties) -> bool {
        (self.if_cut.is_none() || self.if_cut == Some(properties.is_cut))
            && (self.if_alive.is_none() || self.if_alive == Some(properties.is_alive))
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
    fn get_color(
        self,
        aftik_color: Option<AftikColorId>,
        aftik_colors_map: &HashMap<AftikColorId, AftikColorData>,
    ) -> Color {
        let aftik_color_data = || {
            aftik_colors_map
                .get(&aftik_color.unwrap_or_default())
                .unwrap_or(&DEFAULT_COLOR)
        };

        match self {
            ColorSource::Uncolored => WHITE,
            ColorSource::Primary => aftik_color_data().primary_color.into(),
            ColorSource::Secondary => aftik_color_data().secondary_color.into(),
        }
    }
}

pub fn draw_object(
    data: &TextureData,
    properties: &RenderProperties,
    use_wield_offset: bool,
    pos: Vec2,
    aftik_colors_map: &HashMap<AftikColorId, AftikColorData>,
) {
    let mut pos = pos;
    if use_wield_offset {
        pos.y += data.wield_offset.y;
        pos.x += match properties.direction {
            Direction::Left => -data.wield_offset.x,
            Direction::Right => data.wield_offset.x,
        }
    }
    for layer in &data.layers {
        layer.draw(pos, properties, aftik_colors_map);
    }
}

pub fn get_rect_for_object(
    object_data: &ObjectRenderData,
    assets: &mut RenderAssets,
    pos: Vec2,
) -> Rect {
    let data = assets
        .object_textures
        .lookup_texture(&object_data.texture_type);
    data.layers
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

#[derive(Deserialize)]
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
    texture_type: &BackgroundType,
    offset: Coord,
    camera_space: Rect,
    assets: &RenderAssets,
) {
    let offset = offset as f32 * 120.;
    match assets.lookup_background(texture_type).texture {
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
    fixed: bool,
    #[serde(default)]
    if_cut: Option<bool>,
    #[serde(default)]
    if_alive: Option<bool>,
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
            directional: !self.fixed,
            if_cut: self.if_cut,
            if_alive: self.if_alive,
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
        backgrounds: load_backgrounds()?,
        object_textures: objects::prepare()?,
        aftik_colors: load_aftik_color_data()?,
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
        .get(&BackgroundType::blank())
        .ok_or(Error::MissingBlankBackground)?;

    Ok(backgrounds)
}

mod objects {
    use super::{load_texture_data, Error, LazilyLoadedObjectTextures, TextureData};
    use crate::view::area::TextureType;
    use std::collections::HashMap;

    pub fn prepare() -> Result<LazilyLoadedObjectTextures, Error> {
        let mut textures = HashMap::new();

        load(&mut textures, TextureType::unknown())?;
        load(&mut textures, TextureType::small_unknown())?;

        Ok(LazilyLoadedObjectTextures {
            loaded_textures: textures,
        })
    }

    fn load(
        objects: &mut HashMap<TextureType, TextureData>,
        texture_type: TextureType,
    ) -> Result<(), Error> {
        let data = load_texture_data(texture_type.path())?;
        objects.insert(texture_type, data);
        Ok(())
    }

    pub fn load_or_default(
        objects: &mut HashMap<TextureType, TextureData>,
        texture_type: &TextureType,
    ) {
        let path = texture_type.path();
        let texture_data = load_texture_data(path).unwrap_or_else(|error| {
            eprintln!("Unable to load texture data \"{path}\": {error}");
            if texture_type.path().starts_with("item/") {
                objects.get(&TextureType::small_unknown()).unwrap().clone()
            } else {
                objects.get(&TextureType::unknown()).unwrap().clone()
            }
        });
        objects.insert(texture_type.clone(), texture_data);
    }
}

pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorData>, Error> {
    let file = File::open("assets/aftik_colors.json")?;
    Ok(serde_json::from_reader::<
        _,
        HashMap<AftikColorId, AftikColorData>,
    >(file)?)
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
