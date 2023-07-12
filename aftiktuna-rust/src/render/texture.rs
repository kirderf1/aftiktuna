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
    fn new_static(texture: Texture2D) -> TextureData {
        TextureData {
            texture,
            dest_size: Vec2::new(texture.width(), texture.height()),
            directional: false,
        }
    }
    fn new_directional(texture: Texture2D) -> TextureData {
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

fn texture_path(name: &str) -> String {
    format!("assets/textures/{}.png", name)
}

pub async fn load_textures() -> TextureStorage {
    let forest_background = load_texture(&texture_path("tree_background"))
        .await
        .unwrap();
    let blank_background = load_texture(&texture_path("white_space")).await.unwrap();
    let selection_background = load_texture(&texture_path("selection_background"))
        .await
        .unwrap();
    let unknown = load_texture(&texture_path("unknown")).await.unwrap();
    let door = load_texture(&texture_path("door")).await.unwrap();
    let path = load_texture(&texture_path("path")).await.unwrap();
    let aftik = load_texture(&texture_path("aftik")).await.unwrap();
    let goblin = load_texture(&texture_path("goblin")).await.unwrap();
    let eyesaur = load_texture(&texture_path("eyesaur")).await.unwrap();
    let azureclops = load_texture(&texture_path("azureclops")).await.unwrap();
    let fuel_can = load_texture(&texture_path("fuel_can")).await.unwrap();
    let crowbar = load_texture(&texture_path("crowbar")).await.unwrap();
    let knife = load_texture(&texture_path("knife")).await.unwrap();
    let bat = load_texture(&texture_path("bat")).await.unwrap();
    let sword = load_texture(&texture_path("sword")).await.unwrap();

    let mut backgrounds = HashMap::new();

    backgrounds.insert(
        BGTextureType::LocationChoice,
        BGTexture::Simple(selection_background),
    );
    backgrounds.insert(
        BGTextureType::Forest,
        BGTexture::Repeating(forest_background),
    );
    backgrounds.insert(BGTextureType::Blank, BGTexture::Simple(blank_background));

    let mut objects = HashMap::new();

    objects.insert(TextureType::Unknown, TextureData::new_static(unknown));
    objects.insert(
        TextureType::SmallUnknown,
        TextureData {
            texture: unknown,
            dest_size: Vec2::new(100., 100.),
            directional: false,
        },
    );
    objects.insert(TextureType::Door, TextureData::new_static(door));
    objects.insert(TextureType::Path, TextureData::new_static(path));
    objects.insert(TextureType::Aftik, TextureData::new_directional(aftik));
    objects.insert(TextureType::Goblin, TextureData::new_directional(goblin));
    objects.insert(TextureType::Eyesaur, TextureData::new_directional(eyesaur));
    objects.insert(
        TextureType::Azureclops,
        TextureData::new_directional(azureclops),
    );
    objects.insert(
        TextureType::Item(item::Type::FuelCan),
        TextureData::new_static(fuel_can),
    );
    objects.insert(
        TextureType::Item(item::Type::Crowbar),
        TextureData::new_static(crowbar),
    );
    objects.insert(
        TextureType::Item(item::Type::Knife),
        TextureData::new_static(knife),
    );
    objects.insert(
        TextureType::Item(item::Type::Bat),
        TextureData::new_static(bat),
    );
    objects.insert(
        TextureType::Item(item::Type::Sword),
        TextureData::new_static(sword),
    );

    TextureStorage {
        backgrounds,
        objects,
    }
}
