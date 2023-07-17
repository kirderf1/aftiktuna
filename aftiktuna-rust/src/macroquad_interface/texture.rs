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
        directional: bool,
    },
    Mounted {
        texture: Texture2D,
        offset: f32,
    },
    Aftik {
        primary: Texture2D,
        secondary: Texture2D,
        details: Texture2D,
    },
}

impl TextureData {
    async fn load_static(path: &str) -> Result<TextureData, FileError> {
        let texture = load_texture(path).await?;
        Ok(Self::new_static(texture))
    }

    fn new_static(texture: Texture2D) -> TextureData {
        TextureData::Regular {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: false,
        }
    }

    async fn load_directional(path: &str) -> Result<TextureData, FileError> {
        let texture = load_texture(path).await?;
        Ok(TextureData::Regular {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: true,
        })
    }

    async fn load_door(path: &str) -> Result<TextureData, FileError> {
        Self::load_mounted(path, 30.).await
    }

    async fn load_mounted(path: &str, offset: f32) -> Result<TextureData, FileError> {
        let texture = load_texture(path).await?;
        Ok(TextureData::Mounted {
            texture,
            offset,
        })
    }

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
        match self {
            TextureData::Mounted {..} => false,
            _ => true,
        }
    }
}

pub fn draw_object(data: &ObjectRenderData, textures: &TextureStorage, pos: Vec2) {
    match textures.lookup_texture(data.texture_type) {
        TextureData::Regular {
            texture,
            dest_size,
            directional,
        } => {
            draw_texture_ex(
                *texture,
                pos.x - dest_size.x / 2.,
                pos.y - dest_size.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(*dest_size),
                    flip_x: *directional && data.direction == Direction::Left,
                    ..Default::default()
                },
            );
        }
        TextureData::Mounted {
            texture, offset,
        } => {
            draw_texture(
                *texture,
                pos.x - texture.width() / 2.,
                pos.y - texture.height() - offset,
                WHITE,
            );
        }
        TextureData::Aftik {
            primary,
            secondary,
            details,
        } => {
            let params = DrawTextureParams {
                flip_x: data.direction == Direction::Left,
                ..Default::default()
            };
            let (primary_color, secondary_color) =
                convert_to_color(data.aftik_color.unwrap_or(AftikColor::Mint));
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
        TextureData::Regular { dest_size, .. } => Rect::new(
            pos.x - dest_size.x / 2.,
            pos.y - dest_size.y,
            dest_size.x,
            dest_size.y,
        ),
        TextureData::Mounted {texture, offset} => Rect::new(
            pos.x - texture.width() / 2.,
            pos.y - texture.height() - offset,
            texture.width(),
            texture.height(),
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
    let mut backgrounds = HashMap::new();

    backgrounds.insert(
        BGTextureType::LocationChoice,
        BGTexture::centered("location_choice").await?,
    );
    backgrounds.insert(
        BGTextureType::Blank,
        BGTexture::centered("white_space").await?,
    );
    backgrounds.insert(
        BackgroundType::Ship.into(),
        BGTexture::centered("ship").await?,
    );
    backgrounds.insert(
        BackgroundType::ForestEntrance.into(),
        BGTexture::repeating("forest_entrance").await?,
    );
    backgrounds.insert(
        BackgroundType::Forest.into(),
        BGTexture::repeating("forest").await?,
    );
    backgrounds.insert(
        BackgroundType::Field.into(),
        BGTexture::repeating("field").await?,
    );
    backgrounds.insert(
        BackgroundType::Shack.into(),
        BGTexture::centered("shack").await?,
    );
    backgrounds.insert(
        BackgroundType::FacilityOutside.into(),
        BGTexture::fixed("facility_outside").await?,
    );

    let mut objects = HashMap::new();

    let unknown_texture = load_texture("unknown").await?;
    objects.insert(
        TextureType::Unknown,
        TextureData::new_static(unknown_texture),
    );
    objects.insert(
        TextureType::SmallUnknown,
        TextureData::Regular {
            texture: unknown_texture,
            dest_size: Vec2::new(100., 100.),
            directional: false,
        },
    );
    objects.insert(TextureType::Ship, TextureData::load_static("ship").await?);
    objects.insert(TextureType::Door, TextureData::load_door("door").await?);
    objects.insert(
        TextureType::ShipExit,
        TextureData::load_door("ship_exit").await?,
    );
    objects.insert(TextureType::Shack, TextureData::load_door("shack").await?);
    objects.insert(TextureType::Path, TextureData::load_mounted("path", 0.).await?);
    objects.insert(TextureType::Aftik, TextureData::load_aftik().await?);
    objects.insert(
        TextureType::Goblin,
        TextureData::load_directional("creature/goblin").await?,
    );
    objects.insert(
        TextureType::Eyesaur,
        TextureData::load_directional("creature/eyesaur").await?,
    );
    objects.insert(
        TextureType::Azureclops,
        TextureData::load_directional("creature/azureclops").await?,
    );
    objects.insert(
        item::Type::FuelCan.into(),
        TextureData::load_static("item/fuel_can").await?,
    );
    objects.insert(
        item::Type::Crowbar.into(),
        TextureData::load_static("item/crowbar").await?,
    );
    objects.insert(
        item::Type::Blowtorch.into(),
        TextureData::load_static("item/blowtorch").await?,
    );
    objects.insert(
        item::Type::Keycard.into(),
        TextureData::load_static("item/keycard").await?,
    );
    objects.insert(
        item::Type::Knife.into(),
        TextureData::load_static("item/knife").await?,
    );
    objects.insert(
        item::Type::Bat.into(),
        TextureData::load_static("item/bat").await?,
    );
    objects.insert(
        item::Type::Sword.into(),
        TextureData::load_static("item/sword").await?,
    );
    objects.insert(
        item::Type::Medkit.into(),
        TextureData::load_static("item/medkit").await?,
    );
    objects.insert(
        item::Type::MeteorChunk.into(),
        TextureData::load_static("item/meteor_chunk").await?,
    );
    objects.insert(
        item::Type::AncientCoin.into(),
        TextureData::load_static("item/ancient_coin").await?,
    );

    Ok(TextureStorage {
        backgrounds,
        objects,
        left_mouse_icon: load_texture("left_mouse").await?,
    })
}
