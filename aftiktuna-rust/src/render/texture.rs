use aftiktuna::area::BackgroundType;
use aftiktuna::item;
use aftiktuna::position::Direction;
use aftiktuna::view::TextureType;
use macroquad::color::WHITE;
use macroquad::math::Vec2;
use macroquad::prelude::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D};
use std::collections::HashMap;

pub struct TextureStorage {
    backgrounds: HashMap<BGTextureType, BGTexture>,
    objects: HashMap<TextureType, TextureData>,
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
    texture: Texture2D,
    dest_size: Vec2,
    directional: bool,
}

impl TextureData {
    async fn new_static(path: &str) -> TextureData {
        let path = texture_path(path);
        let texture = load_texture(&path).await.unwrap();
        TextureData {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: false,
        }
    }
    async fn new_directional(path: &str) -> TextureData {
        let path = texture_path(path);
        let texture = load_texture(&path).await.unwrap();
        TextureData {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: true,
        }
    }
}

pub fn draw_object(data: &TextureData, direction: Direction, x: f32, y: f32) {
    let size = data.dest_size;
    draw_texture_ex(
        data.texture,
        x - size.x / 2.,
        y - size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            flip_x: data.directional && direction == Direction::Left,
            ..Default::default()
        },
    );
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
        let path = texture_path(&format!("background/{}", path));
        let texture = load_texture(&path).await.unwrap();
        BGTexture::Simple(texture)
    }
    async fn repeating(path: &str) -> BGTexture {
        let path = texture_path(&format!("background/{}", path));
        let texture = load_texture(&path).await.unwrap();
        BGTexture::Repeating(texture)
    }
}

fn texture_path(name: &str) -> String {
    format!("assets/texture/{}.png", name)
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

    let unknown = TextureData::new_static("unknown").await;
    let unknown_texture = unknown.texture;
    objects.insert(TextureType::Unknown, unknown);
    objects.insert(
        TextureType::SmallUnknown,
        TextureData {
            texture: unknown_texture,
            dest_size: Vec2::new(100., 100.),
            directional: false,
        },
    );
    objects.insert(TextureType::Door, TextureData::new_static("door").await);
    objects.insert(TextureType::Path, TextureData::new_static("path").await);
    objects.insert(
        TextureType::Aftik,
        TextureData::new_directional("creature/aftik").await,
    );
    objects.insert(
        TextureType::Goblin,
        TextureData::new_directional("creature/goblin").await,
    );
    objects.insert(
        TextureType::Eyesaur,
        TextureData::new_directional("creature/eyesaur").await,
    );
    objects.insert(
        TextureType::Azureclops,
        TextureData::new_directional("creature/azureclops").await,
    );
    objects.insert(
        TextureType::Item(item::Type::FuelCan),
        TextureData::new_static("item/fuel_can").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Crowbar),
        TextureData::new_static("item/crowbar").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Knife),
        TextureData::new_static("item/knife").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Bat),
        TextureData::new_static("item/bat").await,
    );
    objects.insert(
        TextureType::Item(item::Type::Sword),
        TextureData::new_static("item/sword").await,
    );

    TextureStorage {
        backgrounds,
        objects,
    }
}
