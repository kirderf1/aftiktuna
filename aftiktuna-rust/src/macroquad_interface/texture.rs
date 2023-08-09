use crate::area::BackgroundType;
use crate::item;
use crate::position::{Coord, Direction};
use crate::view::{AftikColor, ObjectRenderData, TextureType};
use macroquad::color::WHITE;
use macroquad::math::Vec2;
use macroquad::prelude::{
    draw_texture, draw_texture_ex, Color, DrawTextureParams, FileError, Rect, Texture2D,
};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub struct TextureStorage {
    backgrounds: HashMap<BGTextureType, BGTexture>,
    objects: HashMap<TextureType, TextureData>,
    pub left_mouse_icon: Texture2D,
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
pub enum TextureData {
    Regular {
        texture: Texture2D,
        dest_size: Vec2,
        y_offset: f32,
        wield_offset: Vec2,
        directional: bool,
        is_mounted: bool,
    },
    Aftik {
        primary: Texture2D,
        secondary: Texture2D,
        details: Texture2D,
    },
}

impl TextureData {
    async fn load_aftik() -> Result<TextureData, FileError> {
        async fn texture(suffix: &str) -> Result<Texture2D, FileError> {
            load_texture(format!("creature/aftik_{}", suffix)).await
        }
        Ok(TextureData::Aftik {
            primary: texture("primary").await?,
            secondary: texture("secondary").await?,
            details: texture("details").await?,
        })
    }

    pub fn is_displacing(&self) -> bool {
        !matches!(self, TextureData::Regular { is_mounted, .. } if *is_mounted)
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
        Ok(TextureData::Regular {
            texture,
            dest_size: self
                .dest_size
                .unwrap_or_else(|| Vec2::new(texture.width(), texture.height())),
            y_offset: self.y_offset.unwrap_or(0.),
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
    match textures.lookup_texture(texture_type) {
        TextureData::Regular {
            texture,
            dest_size,
            y_offset,
            wield_offset,
            directional,
            ..
        } => {
            let mut x = pos.x - dest_size.x / 2.;
            let mut y = pos.y - dest_size.y + y_offset;
            if use_wield_offset {
                y += wield_offset.y;
                x += match direction {
                    Direction::Left => -wield_offset.x,
                    Direction::Right => wield_offset.x,
                }
            }
            draw_texture_ex(
                *texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(*dest_size),
                    flip_x: *directional && direction == Direction::Left,
                    ..Default::default()
                },
            );
        }
        TextureData::Aftik {
            primary,
            secondary,
            details,
        } => {
            let params = DrawTextureParams {
                flip_x: direction == Direction::Left,
                ..Default::default()
            };
            let (primary_color, secondary_color) =
                convert_to_color(aftik_color.unwrap_or(AftikColor::Mint));
            draw_texture_ex(
                *primary,
                pos.x - primary.width() / 2.,
                pos.y - primary.height(),
                primary_color,
                params.clone(),
            );
            draw_texture_ex(
                *secondary,
                pos.x - secondary.width() / 2.,
                pos.y - secondary.height(),
                secondary_color,
                params.clone(),
            );
            draw_texture_ex(
                *details,
                pos.x - details.width() / 2.,
                pos.y - details.height(),
                WHITE,
                params,
            );
        }
    }
}

pub fn get_rect_for_object(data: &ObjectRenderData, textures: &TextureStorage, pos: Vec2) -> Rect {
    let texture = textures.lookup_texture(data.texture_type);
    match texture {
        TextureData::Regular {
            dest_size,
            y_offset,
            ..
        } => Rect::new(
            pos.x - dest_size.x / 2.,
            pos.y - dest_size.y + y_offset,
            dest_size.x,
            dest_size.y,
        ),
        TextureData::Aftik { primary, .. } => Rect::new(
            pos.x - primary.width() / 2.,
            pos.y - primary.height(),
            primary.width(),
            primary.height(),
        ),
    }
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
