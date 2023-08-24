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
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub struct TextureStorage {
    backgrounds: HashMap<BGTextureType, BGTexture>,
    objects: HashMap<TextureType, TextureData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
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
    async fn load_aftik() -> Result<TextureData, FileError> {
        async fn texture(suffix: &str) -> Result<Texture2D, FileError> {
            load_texture(format!("creature/aftik_{}", suffix)).await
        }
        Ok(TextureData {
            layers: vec![
                TextureLayer::simple(texture("primary").await?, ColorSource::Primary),
                TextureLayer::simple(texture("secondary").await?, ColorSource::Secondary),
                TextureLayer::simple(texture("details").await?, ColorSource::Uncolored),
            ],
            wield_offset: Vec2::ZERO,
            directional: true,
            is_mounted: false,
        })
    }

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
    fn simple(texture: Texture2D, color: ColorSource) -> Self {
        Self {
            texture,
            color,
            dest_size: Vec2::new(texture.width(), texture.height()),
            y_offset: 0.,
        }
    }

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

#[derive(Copy, Clone)]
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

struct Builder {
    path: String,
    dest_size: Option<Vec2>,
    y_offset: Option<f32>,
    wield_offset: Option<Vec2>,
    directional: bool,
    is_mounted: bool,
}

impl Builder {
    fn new(path: impl Into<String>, directional: bool) -> Self {
        Builder {
            path: path.into(),
            dest_size: None,
            y_offset: None,
            wield_offset: None,
            directional,
            is_mounted: false,
        }
    }

    fn override_size(mut self, x: f32, y: f32) -> Self {
        self.dest_size = Some(Vec2::new(x, y));
        self
    }

    fn wield_offset(mut self, x: f32, y: f32) -> Self {
        self.wield_offset = Some(Vec2::new(x, y));
        self
    }

    fn mounted(mut self, y_offset: f32) -> Self {
        self.y_offset = Some(-y_offset);
        self.is_mounted = true;
        self
    }

    async fn build(self) -> Result<TextureData, FileError> {
        let texture = load_texture(self.path).await?;
        Ok(TextureData {
            layers: vec![TextureLayer {
                texture,
                color: ColorSource::Uncolored,
                dest_size: self
                    .dest_size
                    .unwrap_or_else(|| Vec2::new(texture.width(), texture.height())),
                y_offset: self.y_offset.unwrap_or(0.),
            }],
            wield_offset: self.wield_offset.unwrap_or(Vec2::ZERO),
            directional: self.directional,
            is_mounted: self.is_mounted,
        })
    }
}

pub fn draw_object(
    texture_type: TextureType,
    direction: Direction,
    aftik_color: Option<AftikColor>,
    use_wield_offset: bool,
    textures: &TextureStorage,
    pos: Vec2,
) {
    let data = textures.lookup_texture(texture_type);
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

async fn load_texture(name: impl Borrow<str>) -> Result<Texture2D, FileError> {
    macroquad::texture::load_texture(&format!("assets/texture/{}.png", name.borrow())).await
}

pub async fn load_textures() -> Result<TextureStorage, FileError> {
    Ok(TextureStorage {
        backgrounds: load_backgrounds().await?,
        objects: load_objects().await?,
        left_mouse_icon: load_texture("left_mouse").await?,
        side_arrow: load_texture("side_arrow").await?,
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

async fn load_objects() -> Result<HashMap<TextureType, TextureData>, FileError> {
    let mut objects = HashMap::new();

    objects.insert(
        TextureType::Unknown,
        Builder::new("unknown", false).build().await?,
    );
    objects.insert(
        TextureType::SmallUnknown,
        Builder::new("unknown", false)
            .override_size(100., 100.)
            .build()
            .await?,
    );
    insert_or_log(
        &mut objects,
        TextureType::FortunaChest,
        Builder::new("fortuna_chest", false).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Ship,
        Builder::new("ship", false).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Door,
        Builder::new("door", false).mounted(30.).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::CutDoor,
        Builder::new("cut_door", false).mounted(15.).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::ShipExit,
        Builder::new("ship_exit", false).mounted(30.).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Shack,
        Builder::new("shack", false).mounted(30.).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::CutShack,
        Builder::new("cut_shack", false).mounted(15.).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Path,
        Builder::new("path", false).mounted(0.).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Aftik,
        TextureData::load_aftik().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Goblin,
        Builder::new("creature/goblin", true).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Eyesaur,
        Builder::new("creature/eyesaur", true).build().await,
    );
    insert_or_log(
        &mut objects,
        TextureType::Azureclops,
        Builder::new("creature/azureclops", true).build().await,
    );
    insert_or_log(
        &mut objects,
        item::Type::FuelCan,
        Builder::new("item/fuel_can", true).build().await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Crowbar,
        Builder::new("item/crowbar", true)
            .wield_offset(10., -35.)
            .build()
            .await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Blowtorch,
        Builder::new("item/blowtorch", true).build().await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Keycard,
        Builder::new("item/keycard", true).build().await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Knife,
        Builder::new("item/knife", true)
            .wield_offset(20., -40.)
            .build()
            .await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Bat,
        Builder::new("item/bat", true)
            .wield_offset(30., -30.)
            .build()
            .await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Sword,
        Builder::new("item/sword", true)
            .wield_offset(20., -10.)
            .build()
            .await,
    );
    insert_or_log(
        &mut objects,
        item::Type::Medkit,
        Builder::new("item/medkit", true).build().await,
    );
    insert_or_log(
        &mut objects,
        item::Type::MeteorChunk,
        Builder::new("item/meteor_chunk", true).build().await,
    );
    insert_or_log(
        &mut objects,
        item::Type::AncientCoin,
        Builder::new("item/ancient_coin", true).build().await,
    );
    Ok(objects)
}

fn insert_or_log<K: Eq + Hash, V>(
    objects: &mut HashMap<K, V>,
    key: impl Into<K>,
    result: Result<V, FileError>,
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
