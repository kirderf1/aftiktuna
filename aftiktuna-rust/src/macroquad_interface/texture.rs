use crate::area::BackgroundType;
use crate::core::item;
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
    backgrounds: HashMap<BGTextureType, BGTexture>,
    objects: HashMap<TextureType, TextureData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
    pub portrait: TextureData,
}

impl TextureStorage {
    pub fn lookup_background(&self, texture_type: BGTextureType) -> &BGTexture {
        self.backgrounds
            .get(&texture_type)
            .unwrap_or_else(|| self.backgrounds.get(&BGTextureType::Blank).unwrap())
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

#[derive(Eq, PartialEq, Hash)]
pub enum BGTextureType {
    LocationChoice,
    Blank,
    Background(BackgroundType),
}

impl From<BackgroundType> for BGTextureType {
    fn from(value: BackgroundType) -> Self {
        BGTextureType::Background(value)
    }
}

pub enum BGTexture {
    Centered(Texture2D),
    Fixed(Texture2D),
    Repeating(Texture2D),
}

impl BGTexture {
    async fn centered(path: &str) -> Result<BGTexture, FileError> {
        let texture = load_texture(format!("background/{}", path)).await?;
        Ok(BGTexture::Centered(texture))
    }
    async fn fixed(path: &str) -> Result<BGTexture, FileError> {
        let texture = load_texture(format!("background/{}", path)).await?;
        Ok(BGTexture::Fixed(texture))
    }
    async fn repeating(path: &str) -> Result<BGTexture, FileError> {
        let texture = load_texture(format!("background/{}", path)).await?;
        Ok(BGTexture::Repeating(texture))
    }
}

pub fn draw_background(
    texture_type: BGTextureType,
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
        objects: load_objects().await?,
        left_mouse_icon: load_texture("left_mouse").await?,
        side_arrow: load_texture("side_arrow").await?,
        portrait: load_texture_data("portrait").await?,
    })
}

async fn load_backgrounds() -> Result<HashMap<BGTextureType, BGTexture>, FileError> {
    let mut backgrounds = HashMap::new();

    backgrounds.insert(
        BGTextureType::Blank,
        BGTexture::centered("white_space").await?,
    );
    insert_or_log(
        &mut backgrounds,
        BGTextureType::LocationChoice,
        BGTexture::centered("location_choice").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::Ship,
        BGTexture::centered("ship").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::ForestEntrance,
        BGTexture::repeating("forest_entrance").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::Forest,
        BGTexture::repeating("forest").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::Field,
        BGTexture::repeating("field").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::Shack,
        BGTexture::centered("shack").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::FacilityOutside,
        BGTexture::fixed("facility_outside").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::FacilitySize3,
        BGTexture::centered("3x_facility").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::FacilitySize4,
        BGTexture::centered("4x_facility").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::FacilitySize5,
        BGTexture::centered("5x_facility").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::FacilitySize6,
        BGTexture::centered("6x_facility").await,
    );
    insert_or_log(
        &mut backgrounds,
        BackgroundType::FacilitySize7,
        BGTexture::fixed("7x_facility").await,
    );
    Ok(backgrounds)
}

async fn load_objects() -> Result<HashMap<TextureType, TextureData>, Error> {
    let mut objects = HashMap::new();

    objects.insert(TextureType::Unknown, load_texture_data("unknown").await?);
    objects.insert(
        TextureType::SmallUnknown,
        load_texture_data("small_unknown").await?,
    );
    insert_or_log(
        &mut objects,
        TextureType::FortunaChest,
        load_texture_data("fortuna_chest").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Ship,
        load_texture_data("ship").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Door,
        load_texture_data("door").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::CutDoor,
        load_texture_data("cut_door").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::ShipExit,
        load_texture_data("ship_exit").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Shack,
        load_texture_data("shack").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::CutShack,
        load_texture_data("cut_shack").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Path,
        load_texture_data("path").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Aftik,
        load_texture_data("creature/aftik").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Goblin,
        load_texture_data("creature/goblin").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Eyesaur,
        load_texture_data("creature/eyesaur").await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Azureclops,
        load_texture_data("creature/azureclops").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::FuelCan,
        load_texture_data("item/fuel_can").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Crowbar,
        load_texture_data("item/crowbar").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Blowtorch,
        load_texture_data("item/blowtorch").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Keycard,
        load_texture_data("item/keycard").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Knife,
        load_texture_data("item/knife").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Bat,
        load_texture_data("item/bat").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Sword,
        load_texture_data("item/sword").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Medkit,
        load_texture_data("item/medkit").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::MeteorChunk,
        load_texture_data("item/meteor_chunk").await,
    );
    insert_or_log(
        &mut objects,
        item::Type::AncientCoin,
        load_texture_data("item/ancient_coin").await,
    );
    Ok(objects)
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
