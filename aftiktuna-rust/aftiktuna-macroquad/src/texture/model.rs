use super::Error;
use aftiktuna::asset::color::{AftikColorData, ColorSource};
use aftiktuna::asset::TextureLoader;
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::Direction;
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2};
use macroquad::texture::{self, DrawTextureParams, Texture2D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
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

    pub fn get_rect_for_object(&mut self, object_data: &ObjectRenderData, pos: Vec2) -> Rect {
        let model = self.lookup_model(&object_data.model_id);
        model.get_rect(pos, &object_data.properties)
    }
}

#[derive(Clone)]
pub struct Model {
    layers: Vec<TextureLayer<Texture2D>>,
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
            .filter(|&layer| layer.conditions.meets_conditions(properties))
            .fold(Rect::new(pos.x, pos.y, 0., 0.), |rect, layer| {
                rect.combine_with(layer_render_rect(layer, pos))
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
            draw_layer(layer, pos, properties, aftik_color_data);
        }
    }
}

fn draw_layer(
    layer: &TextureLayer<Texture2D>,
    pos: Vec2,
    properties: &RenderProperties,
    aftik_color_data: &AftikColorData,
) {
    if !layer.conditions.meets_conditions(properties) {
        return;
    }

    let render_rect = layer_render_rect(layer, pos);
    let color = layer.color.get_color(aftik_color_data);
    texture::draw_texture_ex(
        &layer.texture,
        render_rect.x,
        render_rect.y,
        Color::from_rgba(color.r, color.g, color.b, 255),
        DrawTextureParams {
            dest_size: Some(render_rect.size()),
            flip_x: !layer.positioning.fixed && properties.direction == Direction::Left,
            ..Default::default()
        },
    );
}

fn layer_render_rect(layer: &TextureLayer<Texture2D>, pos: Vec2) -> Rect {
    let dest_size = layer
        .positioning
        .size
        .map(|(width, height)| Vec2::new(f32::from(width), f32::from(height)))
        .unwrap_or_else(|| layer.texture.size());
    Rect::new(
        pos.x - dest_size.x / 2.,
        pos.y - dest_size.y + f32::from(layer.positioning.y_offset),
        dest_size.x,
        dest_size.y,
    )
}

#[derive(Serialize, Deserialize)]
pub struct RawModel {
    pub layers: Vec<TextureLayer<String>>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub wield_offset: (i16, i16),
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub mounted: bool,
}

impl RawModel {
    pub fn load<E>(&self, loader: &mut impl TextureLoader<Texture2D, E>) -> Result<Model, E> {
        let mut layers = Vec::new();
        for layer in &self.layers {
            layers.push(layer.load(loader)?);
        }
        layers.reverse();
        Ok(Model {
            layers,
            wield_offset: Vec2::new(
                f32::from(self.wield_offset.0),
                f32::from(self.wield_offset.1),
            ),
            is_mounted: self.mounted,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureLayer<T> {
    pub texture: T,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub color: ColorSource,
    #[serde(flatten)]
    pub positioning: LayerPositioning,
    #[serde(flatten)]
    pub conditions: LayerCondition,
}

impl TextureLayer<String> {
    pub fn texture_path(&self) -> String {
        format!("object/{}", self.texture)
    }

    fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<TextureLayer<T>, E> {
        let texture = loader.load_texture(self.texture_path())?;
        Ok(TextureLayer {
            texture,
            color: self.color,
            positioning: self.positioning.clone(),
            conditions: self.conditions.clone(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayerPositioning {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub size: Option<(i16, i16)>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub y_offset: i16,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub fixed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayerCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_cut: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_hurt: Option<bool>,
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
