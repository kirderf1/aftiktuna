use crate::area::BackgroundType;
use crate::core::position::{Coord, Direction};
use crate::view::{AftikColor, ObjectRenderData, TextureType};
use egui_macroquad::macroquad;
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::file::FileError;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::texture::{
    draw_texture, draw_texture_ex, DrawTextureParams, Texture2D,
};
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io;

pub struct TextureStorage {
    backgrounds: HashMap<BackgroundType, BGTexture>,
    objects: HashMap<TextureType, TextureData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
    pub portrait: TextureData,
}

impl TextureStorage {
    pub fn lookup_background(&self, texture_type: BackgroundType) -> &BGTexture {
        self.backgrounds
            .get(&texture_type)
            .unwrap_or_else(|| self.backgrounds.get(&BackgroundType::Blank).unwrap())
    }

    pub fn lookup_texture(&self, texture_type: TextureType) -> &TextureData {
        if let Some(data) = self.objects.get(&texture_type) {
            data
        } else if let TextureType::Item(_) = texture_type {
            self.objects.get(&TextureType::SmallUnknown).unwrap()
        } else {
            self.objects.get(&TextureType::Unknown).unwrap()
        }
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
}

impl TextureLayer {
    fn draw(&self, pos: Vec2, flip_x: bool, aftik_color: Option<AftikColor>) {
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

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ColorSource {
    Uncolored,
    Primary,
    Secondary,
}

impl ColorSource {
    fn get_color(self, aftik_color: Option<AftikColor>) -> Color {
        match self {
            ColorSource::Uncolored => WHITE,
            ColorSource::Primary => convert_to_color(aftik_color.unwrap_or(AftikColor::Mint)).0,
            ColorSource::Secondary => convert_to_color(aftik_color.unwrap_or(AftikColor::Mint)).1,
        }
    }
}

impl Default for ColorSource {
    fn default() -> Self {
        Self::Uncolored
    }
}

pub fn draw_object(
    data: &TextureData,
    direction: Direction,
    aftik_color: Option<AftikColor>,
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
        );
    }
}

pub fn get_rect_for_object(data: &ObjectRenderData, textures: &TextureStorage, pos: Vec2) -> Rect {
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

pub enum BGTexture {
    Centered(Texture2D),
    Fixed(Texture2D),
    Repeating(Texture2D),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawBGTexture {
    Centered { texture: String },
    Fixed { texture: String },
    Repeating { texture: String },
}

impl RawBGTexture {
    async fn load(self) -> Result<BGTexture, FileError> {
        Ok(match self {
            RawBGTexture::Centered { texture } => {
                BGTexture::Centered(load_texture(format!("background/{texture}")).await?)
            }
            RawBGTexture::Fixed { texture } => {
                BGTexture::Fixed(load_texture(format!("background/{texture}")).await?)
            }
            RawBGTexture::Repeating { texture } => {
                BGTexture::Repeating(load_texture(format!("background/{texture}")).await?)
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
    match textures.lookup_background(texture_type) {
        BGTexture::Centered(texture) => draw_texture(*texture, camera_space.x - offset, 0., WHITE),
        BGTexture::Fixed(texture) => draw_texture(*texture, -60. - offset, 0., WHITE),
        BGTexture::Repeating(texture) => {
            let start_x =
                texture.width() * f32::floor((camera_space.x + offset) / texture.width()) - offset;
            draw_texture(*texture, start_x, 0., WHITE);
            draw_texture(*texture, start_x + texture.width(), 0., WHITE);
        }
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
    async fn load(self) -> Result<TextureData, FileError> {
        let mut layers = Vec::new();
        for layer in self.layers {
            layers.push(layer.load().await?);
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
}

impl RawTextureLayer {
    async fn load(self) -> Result<TextureLayer, FileError> {
        let texture = load_texture(self.texture).await?;
        Ok(TextureLayer {
            texture,
            color: self.color,
            dest_size: Vec2::from(
                self.size
                    .unwrap_or_else(|| (texture.width(), texture.height())),
            ),
            y_offset: self.y_offset,
        })
    }
}

async fn load_texture_data(path: &str) -> Result<TextureData, Error> {
    let file = File::open(format!("assets/texture/{path}.json"))?;
    let data = serde_json::from_reader::<_, RawTextureData>(file)?;
    let data = data.load().await?;
    Ok(data)
}

async fn load_texture(name: impl Borrow<str>) -> Result<Texture2D, FileError> {
    macroquad::texture::load_texture(&format!("assets/texture/{}.png", name.borrow())).await
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

pub async fn load_textures() -> Result<TextureStorage, Error> {
    Ok(TextureStorage {
        backgrounds: load_backgrounds().await?,
        objects: objects::load_all().await?,
        left_mouse_icon: load_texture("left_mouse").await?,
        side_arrow: load_texture("side_arrow").await?,
        portrait: load_texture_data("portrait").await?,
    })
}

async fn load_backgrounds() -> Result<HashMap<BackgroundType, BGTexture>, Error> {
    let file = File::open("assets/texture/background/backgrounds.json")?;
    let raw_backgrounds: HashMap<BackgroundType, RawBGTexture> = serde_json::from_reader(file)?;
    let mut backgrounds = HashMap::new();
    for (bg_type, raw_data) in raw_backgrounds {
        insert_or_log(&mut backgrounds, bg_type, raw_data.load().await);
    }

    backgrounds
        .get(&BackgroundType::Blank)
        .expect("Blank background texture must exist");

    Ok(backgrounds)
}

mod objects {
    use super::{Error, TextureData};
    use crate::core::item;
    use crate::macroquad_interface::texture::{insert_or_log, load_texture_data};
    use crate::view::TextureType;
    use std::collections::HashMap;

    pub async fn load_all() -> Result<HashMap<TextureType, TextureData>, Error> {
        let mut objects = HashMap::new();

        load(&mut objects, TextureType::Unknown, "unknown").await?;
        load(&mut objects, TextureType::SmallUnknown, "small_unknown").await?;
        try_load(&mut objects, TextureType::FortunaChest, "fortuna_chest").await;
        try_load(&mut objects, TextureType::Ship, "ship").await;
        try_load(&mut objects, TextureType::Door, "door").await;
        try_load(&mut objects, TextureType::CutDoor, "cut_door").await;
        try_load(&mut objects, TextureType::ShipExit, "ship_exit").await;
        try_load(&mut objects, TextureType::Shack, "shack").await;
        try_load(&mut objects, TextureType::CutShack, "cut_shack").await;
        try_load(&mut objects, TextureType::Path, "path").await;
        try_load(&mut objects, TextureType::Aftik, "creature/aftik").await;
        try_load(&mut objects, TextureType::Goblin, "creature/goblin").await;
        try_load(&mut objects, TextureType::Eyesaur, "creature/eyesaur").await;
        try_load(&mut objects, TextureType::Azureclops, "creature/azureclops").await;
        try_load(&mut objects, TextureType::Scarvie, "creature/scarvie").await;
        try_load(
            &mut objects,
            TextureType::VoraciousFrog,
            "creature/voracious_frog",
        )
        .await;
        try_load(&mut objects, item::Type::FuelCan, "item/fuel_can").await;
        try_load(&mut objects, item::Type::Crowbar, "item/crowbar").await;
        try_load(&mut objects, item::Type::Blowtorch, "item/blowtorch").await;
        try_load(&mut objects, item::Type::Keycard, "item/keycard").await;
        try_load(&mut objects, item::Type::Knife, "item/knife").await;
        try_load(&mut objects, item::Type::Bat, "item/bat").await;
        try_load(&mut objects, item::Type::Sword, "item/sword").await;
        try_load(&mut objects, item::Type::Medkit, "item/medkit").await;
        try_load(&mut objects, item::Type::MeteorChunk, "item/meteor_chunk").await;
        try_load(&mut objects, item::Type::AncientCoin, "item/ancient_coin").await;

        Ok(objects)
    }

    async fn load(
        objects: &mut HashMap<TextureType, TextureData>,
        key: impl Into<TextureType>,
        path: &str,
    ) -> Result<(), Error> {
        objects.insert(key.into(), load_texture_data(path).await?);
        Ok(())
    }

    async fn try_load(
        objects: &mut HashMap<TextureType, TextureData>,
        key: impl Into<TextureType>,
        path: &str,
    ) {
        insert_or_log(objects, key, load_texture_data(path).await);
    }
}

fn insert_or_log<K: Eq + Hash, V, D: Display>(
    objects: &mut HashMap<K, V>,
    key: impl Into<K>,
    result: Result<V, D>,
) {
    match result {
        Ok(value) => {
            objects.insert(key.into(), value);
        }
        Err(error) => {
            eprintln!("Unable to load texture: {error}")
        }
    }
}
