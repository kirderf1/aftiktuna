use egui_macroquad::macroquad::color::{self, Color};
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::texture::{self, DrawTextureParams, Texture2D};
use serde::{Deserialize, Serialize};

use super::{AftikColorData, Error, TextureLoader};
use crate::core::position::Direction;
use crate::core::ModelId;
use crate::view::area::RenderProperties;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

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

#[derive(Clone)]
pub struct Model {
    layers: Vec<TextureLayer>,
    wield_offset: Vec2,
    is_mounted: bool,
}

impl Model {
    pub fn is_displacing(&self) -> bool {
        !self.is_mounted
    }

    pub fn get_rect(&self, pos: Vec2, properties: &RenderProperties) -> Rect {
        self.layers
            .iter()
            .filter(|&layer| layer.condition.meets_conditions(properties))
            .fold(Rect::new(pos.x, pos.y, 0., 0.), |rect, layer| {
                rect.combine_with(layer.size(pos))
            })
    }

    pub fn draw(
        &self,
        pos: Vec2,
        use_wield_offset: bool,
        properties: &RenderProperties,
        aftik_color_data: &AftikColorData,
    ) {
        let mut pos = pos;
        if use_wield_offset {
            pos.y += self.wield_offset.y;
            pos.x += match properties.direction {
                Direction::Left => -self.wield_offset.x,
                Direction::Right => self.wield_offset.x,
            }
        }
        for layer in &self.layers {
            layer.draw(pos, properties, aftik_color_data);
        }
    }
}

#[derive(Clone)]
struct TextureLayer {
    texture: Texture2D,
    color: ColorSource,
    positioning: LayerPositioning,
    condition: LayerCondition,
}

impl TextureLayer {
    fn draw(&self, pos: Vec2, properties: &RenderProperties, aftik_color_data: &AftikColorData) {
        if !self.condition.meets_conditions(properties) {
            return;
        }

        let dest_size = self.positioning.dest_size(self.texture);
        let x = pos.x - dest_size.x / 2.;
        let y = pos.y + self.positioning.y_offset - dest_size.y;
        texture::draw_texture_ex(
            self.texture,
            x,
            y,
            self.color.get_color(aftik_color_data),
            DrawTextureParams {
                dest_size: Some(dest_size),
                flip_x: !self.positioning.fixed && properties.direction == Direction::Left,
                ..Default::default()
            },
        );
    }

    fn size(&self, pos: Vec2) -> Rect {
        let dest_size = self.positioning.dest_size(self.texture);
        Rect::new(
            pos.x - dest_size.x / 2.,
            pos.y - dest_size.y + self.positioning.y_offset,
            dest_size.x,
            dest_size.y,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct RawModel {
    pub layers: Vec<RawTextureLayer>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub wield_offset: (f32, f32),
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub mounted: bool,
}

impl RawModel {
    pub fn load(&self, loader: &mut impl TextureLoader) -> Result<Model, io::Error> {
        let mut layers = Vec::new();
        for layer in &self.layers {
            layers.push(layer.load(loader)?);
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
pub struct RawTextureLayer {
    pub texture: String,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub color: ColorSource,
    #[serde(flatten)]
    pub positioning: LayerPositioning,
    #[serde(flatten)]
    pub conditions: LayerCondition,
}

impl RawTextureLayer {
    pub fn texture_path(&self) -> String {
        format!("object/{}", self.texture)
    }

    fn load(&self, loader: &mut impl TextureLoader) -> Result<TextureLayer, io::Error> {
        let texture = loader.load_texture(self.texture_path())?;
        Ok(TextureLayer {
            texture,
            color: self.color,
            positioning: self.positioning.clone(),
            condition: self.conditions.clone(),
        })
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorSource {
    #[default]
    Uncolored,
    Primary,
    Secondary,
}

impl ColorSource {
    fn get_color(self, aftik_color_data: &AftikColorData) -> Color {
        match self {
            ColorSource::Uncolored => color::WHITE,
            ColorSource::Primary => aftik_color_data.primary_color.into(),
            ColorSource::Secondary => aftik_color_data.secondary_color.into(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayerPositioning {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub size: Option<(f32, f32)>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub y_offset: f32,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub fixed: bool,
}

impl LayerPositioning {
    fn dest_size(&self, texture: Texture2D) -> Vec2 {
        self.size
            .unwrap_or_else(|| (texture.width(), texture.height()))
            .into()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayerCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    if_cut: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    if_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    if_hurt: Option<bool>,
}

impl LayerCondition {
    fn meets_conditions(&self, properties: &RenderProperties) -> bool {
        (self.if_cut.is_none() || self.if_cut == Some(properties.is_cut))
            && (self.if_alive.is_none() || self.if_alive == Some(properties.is_alive))
            && (self.if_hurt.is_none() || self.if_hurt == Some(properties.is_badly_hurt))
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

fn load_and_insert(model_id: ModelId, models: &mut HashMap<ModelId, Model>) -> Result<(), Error> {
    let model = load_model(&model_id)?;
    models.insert(model_id, model);
    Ok(())
}

fn load_and_insert_or_default(model_id: &ModelId, models: &mut HashMap<ModelId, Model>) {
    let texture_data = load_model(model_id).unwrap_or_else(|error| {
        let path = model_id.path();
        eprintln!("Unable to load texture data \"{path}\": {error}");
        if model_id.path().starts_with("item/") {
            models.get(&ModelId::small_unknown()).unwrap().clone()
        } else {
            models.get(&ModelId::unknown()).unwrap().clone()
        }
    });
    models.insert(model_id.clone(), texture_data);
}

pub fn load_model(model_id: &ModelId) -> Result<Model, Error> {
    Ok(load_raw_model_from_path(model_id.file_path())?.load(&mut super::InPlaceLoader)?)
}

pub fn load_raw_model_from_path(file_path: impl AsRef<Path>) -> Result<RawModel, Error> {
    let file = File::open(file_path)?;
    Ok(serde_json::from_reader::<_, RawModel>(file)?)
}
