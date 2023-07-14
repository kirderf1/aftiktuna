use aftiktuna::area::BackgroundType;
use aftiktuna::item;
use aftiktuna::position::Direction;
use aftiktuna::view::{AftikColor, TextureType};
use macroquad::color::WHITE;
use macroquad::math::Vec2;
use macroquad::prelude::{draw_texture_ex, Color, DrawTextureParams, Texture2D};
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
    Aftik {
        primary: Texture2D,
        secondary: Texture2D,
        details: Texture2D,
    },
}

impl TextureData {
    async fn load_static(path: &str) -> TextureData {
        let texture = load_texture(path).await;
        Self::new_static(texture)
    }

    fn new_static(texture: Texture2D) -> TextureData {
        TextureData::Regular {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: false,
        }
    }

    async fn load_directional(path: &str) -> TextureData {
        let texture = load_texture(path).await;
        TextureData::Regular {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: true,
        }
    }
    async fn load_aftik() -> TextureData {
        async fn texture(suffix: &str) -> Texture2D {
            load_texture(format!("creature/aftik_{}", suffix)).await
        }
        TextureData::Aftik {
            primary: texture("primary").await,
            secondary: texture("secondary").await,
            details: texture("details").await,
        }
    }
}

pub fn draw_object(
    data: &TextureData,
    direction: Direction,
    color: Option<AftikColor>,
    x: f32,
    y: f32,
) {
    match data {
        TextureData::Regular {
            texture,
            dest_size,
            directional,
        } => {
            draw_texture_ex(
                *texture,
                x - dest_size.x / 2.,
                y - dest_size.y,
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
                convert_to_color(color.unwrap_or(AftikColor::Mint));
            draw_texture_ex(
                *primary,
                x - primary.width() / 2.,
                y - primary.height(),
                primary_color,
                params.clone(),
            );
            draw_texture_ex(
                *secondary,
                x - secondary.width() / 2.,
                y - secondary.height(),
                secondary_color,
                params.clone(),
            );
            draw_texture_ex(
                *details,
                x - details.width() / 2.,
                y - details.height(),
                WHITE,
                params,
            );
        }
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
    Forest,
    Blank,
}

impl From<BackgroundType> for BGTextureType {
    fn from(value: BackgroundType) -> Self {
        match value {
            BackgroundType::Forest => BGTextureType::Forest,
        }
    }
}

pub enum BGTexture {
    Simple(Texture2D),
    Repeating(Texture2D),
}

impl BGTexture {
    async fn simple(path: &str) -> BGTexture {
        BGTexture::Simple(load_texture(format!("background/{}", path)).await)
    }
    async fn repeating(path: &str) -> BGTexture {
        BGTexture::Repeating(load_texture(format!("background/{}", path)).await)
    }
}

async fn load_texture(name: impl Borrow<str>) -> Texture2D {
    macroquad::texture::load_texture(&format!("assets/texture/{}.png", name.borrow()))
        .await
        .unwrap()
}

pub async fn load_textures() -> TextureStorage {
    let mut backgrounds = HashMap::new();

    backgrounds.insert(
        BGTextureType::LocationChoice,
        BGTexture::simple("location_choice").await,
    );
    backgrounds.insert(BGTextureType::Forest, BGTexture::repeating("forest").await);
    backgrounds.insert(BGTextureType::Blank, BGTexture::simple("white_space").await);

    let mut objects = HashMap::new();

    let unknown_texture = load_texture("unknown").await;
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
    objects.insert(TextureType::Door, TextureData::load_static("door").await);
    objects.insert(TextureType::Path, TextureData::load_static("path").await);
    objects.insert(TextureType::Aftik, TextureData::load_aftik().await);
    objects.insert(
        TextureType::Goblin,
        TextureData::load_directional("creature/goblin").await,
    );
    objects.insert(
        TextureType::Eyesaur,
        TextureData::load_directional("creature/eyesaur").await,
    );
    objects.insert(
        TextureType::Azureclops,
        TextureData::load_directional("creature/azureclops").await,
    );
    objects.insert(
        TextureType::Item(item::Type::FuelCan),
        TextureData::load_static("item/fuel_can").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Crowbar),
        TextureData::load_static("item/crowbar").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Knife),
        TextureData::load_static("item/knife").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Bat),
        TextureData::load_static("item/bat").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Sword),
        TextureData::load_static("item/sword").await,
    );

    TextureStorage {
        backgrounds,
        objects,
        left_mouse_icon: load_texture("left_mouse").await,
    }
}
