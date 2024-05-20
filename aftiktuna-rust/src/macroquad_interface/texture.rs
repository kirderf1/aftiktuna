use crate::core::area::BackgroundType;
use crate::core::position::{Coord, Direction};
use crate::view::area::RenderProperties;
use crate::view::area::{AftikColorId, ModelId, ObjectRenderData};
use egui_macroquad::macroquad::color::{Color, WHITE};
use egui_macroquad::macroquad::file::FileError;
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::prelude::ImageFormat;
use egui_macroquad::macroquad::texture::{draw_texture, Texture2D};
use egui_macroquad::macroquad::window;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Read;

use self::background::{BGData, BGPortrait, BGTexture};
pub use self::model::LazilyLoadedModels;

pub struct RenderAssets {
    backgrounds: HashMap<BackgroundType, BGData>,
    pub models: LazilyLoadedModels,
    aftik_colors: HashMap<AftikColorId, AftikColorData>,
    pub left_mouse_icon: Texture2D,
    pub side_arrow: Texture2D,
}

impl RenderAssets {
    pub fn lookup_background(&self, texture_type: &BackgroundType) -> &BGData {
        self.backgrounds
            .get(texture_type)
            .unwrap_or_else(|| self.backgrounds.get(&BackgroundType::blank()).unwrap())
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
        aftik_color: Option<&AftikColorId>,
        aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
    ) -> Color {
        let mut aftik_color_data = || {
            aftik_color.map_or(DEFAULT_COLOR, |aftik_color| {
                lookup_or_log_aftik_color(aftik_color, aftik_colors_map)
            })
        };

        match self {
            ColorSource::Uncolored => WHITE,
            ColorSource::Primary => aftik_color_data().primary_color.into(),
            ColorSource::Secondary => aftik_color_data().secondary_color.into(),
        }
    }
}

fn lookup_or_log_aftik_color(
    aftik_color: &AftikColorId,
    aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
) -> AftikColorData {
    match aftik_colors_map.get(aftik_color) {
        Some(color_data) => color_data.clone(),
        None => {
            eprintln!("Missing aftik color data for color {aftik_color:?}!");
            aftik_colors_map.insert(aftik_color.clone(), DEFAULT_COLOR);
            DEFAULT_COLOR
        }
    }
}

pub fn draw_object(
    model_id: &ModelId,
    properties: &RenderProperties,
    use_wield_offset: bool,
    pos: Vec2,
    assets: &mut RenderAssets,
) {
    let model = assets.models.lookup_model(model_id);
    let mut pos = pos;
    if use_wield_offset {
        pos.y += model.wield_offset.y;
        pos.x += match properties.direction {
            Direction::Left => -model.wield_offset.x,
            Direction::Right => model.wield_offset.x,
        }
    }
    for layer in &model.layers {
        layer.draw(pos, properties, &mut assets.aftik_colors);
    }
}

pub fn get_rect_for_object(
    object_data: &ObjectRenderData,
    assets: &mut RenderAssets,
    pos: Vec2,
) -> Rect {
    let model = assets.models.lookup_model(&object_data.texture_type);
    model
        .layers
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

#[derive(Clone, Deserialize)]
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

pub fn draw_background_portrait(background_type: &BackgroundType, assets: &RenderAssets) {
    match assets.lookup_background(background_type).portrait {
        BGPortrait::Color(color) => window::clear_background(color),
        BGPortrait::Texture(texture) => draw_texture(texture, 0., 0., WHITE),
    }
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
        backgrounds: background::load_backgrounds()?,
        models: model::prepare()?,
        aftik_colors: load_aftik_color_data()?,
        left_mouse_icon: load_texture("left_mouse")?,
        side_arrow: load_texture("side_arrow")?,
    })
}

mod background {
    use std::collections::HashMap;
    use std::fmt::Display;
    use std::fs::File;
    use std::hash::Hash;
    use std::io;

    use egui_macroquad::macroquad::color::Color;
    use egui_macroquad::macroquad::texture::Texture2D;
    use serde::{Deserialize, Serialize};

    use crate::core::area::BackgroundType;

    pub fn load_backgrounds() -> Result<HashMap<BackgroundType, BGData>, super::Error> {
        let file = File::open("assets/texture/background/backgrounds.json")?;
        let raw_backgrounds: HashMap<BackgroundType, RawBGData> = serde_json::from_reader(file)?;
        let mut backgrounds = HashMap::new();
        for (bg_type, raw_data) in raw_backgrounds {
            insert_or_log(&mut backgrounds, bg_type, raw_data.load());
        }

        backgrounds
            .get(&BackgroundType::blank())
            .ok_or(super::Error::MissingBlankBackground)?;

        Ok(backgrounds)
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

    pub struct BGData {
        pub texture: BGTexture,
        pub portrait: BGPortrait,
    }

    pub enum BGTexture {
        Centered(Texture2D),
        Fixed(Texture2D),
        Repeating(Texture2D),
    }

    pub enum BGPortrait {
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
                    BGTexture::Centered(super::load_texture(format!("background/{texture}"))?)
                }
                RawBGTexture::Fixed { texture } => {
                    BGTexture::Fixed(super::load_texture(format!("background/{texture}"))?)
                }
                RawBGTexture::Repeating { texture } => {
                    BGTexture::Repeating(super::load_texture(format!("background/{texture}"))?)
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
                    BGPortrait::Texture(super::load_texture(format!("background/{texture}"))?)
                }
            })
        }
    }
}

mod model {
    use egui_macroquad::macroquad::math::{Rect, Vec2};
    use egui_macroquad::macroquad::texture::{self, DrawTextureParams, Texture2D};
    use serde::{Deserialize, Serialize};

    use super::{AftikColorData, ColorSource, Error};
    use crate::core::position::Direction;
    use crate::view::area::{AftikColorId, ModelId, RenderProperties};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io;

    pub struct LazilyLoadedModels {
        loaded_models: HashMap<ModelId, Model>,
    }

    impl LazilyLoadedModels {
        pub fn lookup_model(&mut self, model_id: &ModelId) -> &Model {
            if !self.loaded_models.contains_key(model_id) {
                load_and_insert_or_default(model_id, &mut self.loaded_models);
            }
            self.loaded_models.get(model_id).unwrap()
        }
    }

    pub fn prepare() -> Result<LazilyLoadedModels, Error> {
        let mut models = HashMap::new();

        load_and_insert(ModelId::unknown(), &mut models)?;
        load_and_insert(ModelId::small_unknown(), &mut models)?;

        Ok(LazilyLoadedModels {
            loaded_models: models,
        })
    }

    fn load_and_insert(
        model_id: ModelId,
        models: &mut HashMap<ModelId, Model>,
    ) -> Result<(), Error> {
        let model = load_model(model_id.path())?;
        models.insert(model_id, model);
        Ok(())
    }

    fn load_and_insert_or_default(model_id: &ModelId, models: &mut HashMap<ModelId, Model>) {
        let path = model_id.path();
        let texture_data = load_model(path).unwrap_or_else(|error| {
            eprintln!("Unable to load texture data \"{path}\": {error}");
            if model_id.path().starts_with("item/") {
                models.get(&ModelId::small_unknown()).unwrap().clone()
            } else {
                models.get(&ModelId::unknown()).unwrap().clone()
            }
        });
        models.insert(model_id.clone(), texture_data);
    }

    fn load_model(path: &str) -> Result<Model, Error> {
        let file = File::open(format!("assets/texture/object/{path}.json"))?;
        let model = serde_json::from_reader::<_, RawModel>(file)?;
        Ok(model.load()?)
    }

    #[derive(Clone)]
    pub struct Model {
        pub layers: Vec<TextureLayer>,
        pub wield_offset: Vec2,
        is_mounted: bool,
    }

    impl Model {
        pub fn is_displacing(&self) -> bool {
            !self.is_mounted
        }
    }

    #[derive(Clone)]
    pub struct TextureLayer {
        texture: Texture2D,
        color: ColorSource,
        dest_size: Vec2,
        y_offset: f32,
        directional: bool,
        if_cut: Option<bool>,
        if_alive: Option<bool>,
    }

    impl TextureLayer {
        pub fn draw(
            &self,
            pos: Vec2,
            properties: &RenderProperties,
            aftik_colors_map: &mut HashMap<AftikColorId, AftikColorData>,
        ) {
            if !self.is_active(properties) {
                return;
            }

            let x = pos.x - self.dest_size.x / 2.;
            let y = pos.y + self.y_offset - self.dest_size.y;
            texture::draw_texture_ex(
                self.texture,
                x,
                y,
                self.color
                    .get_color(properties.aftik_color.as_ref(), aftik_colors_map),
                DrawTextureParams {
                    dest_size: Some(self.dest_size),
                    flip_x: self.directional && properties.direction == Direction::Left,
                    ..Default::default()
                },
            );
        }

        pub fn size(&self, pos: Vec2) -> Rect {
            Rect::new(
                pos.x - self.dest_size.x / 2.,
                pos.y - self.dest_size.y + self.y_offset,
                self.dest_size.x,
                self.dest_size.y,
            )
        }

        pub fn is_active(&self, properties: &RenderProperties) -> bool {
            (self.if_cut.is_none() || self.if_cut == Some(properties.is_cut))
                && (self.if_alive.is_none() || self.if_alive == Some(properties.is_alive))
        }
    }

    #[derive(Serialize, Deserialize)]
    struct RawModel {
        layers: Vec<RawTextureLayer>,
        #[serde(default)]
        wield_offset: (f32, f32),
        #[serde(default)]
        mounted: bool,
    }

    impl RawModel {
        fn load(self) -> Result<Model, io::Error> {
            let mut layers = Vec::new();
            for layer in self.layers {
                layers.push(layer.load()?);
            }
            layers.reverse();
            Ok(Model {
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
            let texture = super::load_texture(format!("object/{}", self.texture))?;
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
}

pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorData>, Error> {
    let file = File::open("assets/aftik_colors.json")?;
    Ok(serde_json::from_reader::<
        _,
        HashMap<AftikColorId, AftikColorData>,
    >(file)?)
}
