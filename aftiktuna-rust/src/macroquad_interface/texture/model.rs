use egui_macroquad::macroquad::color::{self, Color};
use egui_macroquad::macroquad::math::{Rect, Vec2};
use egui_macroquad::macroquad::texture::{self, DrawTextureParams, Texture2D};
use serde::{Deserialize, Serialize};

use super::{AftikColorData, Error};
use crate::core::position::Direction;
use crate::core::ModelId;
use crate::view::area::RenderProperties;
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
    dest_size: Vec2,
    y_offset: f32,
    directional: bool,
    condition: LayerCondition,
}

impl TextureLayer {
    fn draw(&self, pos: Vec2, properties: &RenderProperties, aftik_color_data: &AftikColorData) {
        if !self.condition.meets_conditions(properties) {
            return;
        }

        let x = pos.x - self.dest_size.x / 2.;
        let y = pos.y + self.y_offset - self.dest_size.y;
        texture::draw_texture_ex(
            self.texture,
            x,
            y,
            self.color.get_color(aftik_color_data),
            DrawTextureParams {
                dest_size: Some(self.dest_size),
                flip_x: self.directional && properties.direction == Direction::Left,
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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
    #[serde(flatten)]
    conditions: LayerCondition,
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
            condition: self.conditions,
        })
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
    fn get_color(self, aftik_color_data: &AftikColorData) -> Color {
        match self {
            ColorSource::Uncolored => color::WHITE,
            ColorSource::Primary => aftik_color_data.primary_color.into(),
            ColorSource::Secondary => aftik_color_data.secondary_color.into(),
        }
    }
}

#[derive(Clone, Deserialize)]
struct LayerCondition {
    #[serde(default)]
    if_cut: Option<bool>,
    #[serde(default)]
    if_alive: Option<bool>,
    #[serde(default)]
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

pub fn load_model(ModelId(path): &ModelId) -> Result<Model, Error> {
    let file = File::open(format!("assets/texture/object/{path}.json"))?;
    let model = serde_json::from_reader::<_, RawModel>(file)?;
    Ok(model.load()?)
}