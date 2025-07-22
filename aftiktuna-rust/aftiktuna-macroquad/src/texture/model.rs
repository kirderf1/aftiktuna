use super::Error;
use aftiktuna::asset::color::AftikColorData;
use aftiktuna::asset::model::{self, Model, ModelAccess, TextureLayer};
use aftiktuna::core::display::ModelId;
use aftiktuna::core::position::Direction;
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2};
use macroquad::texture::{self, DrawTextureParams, Texture2D};
use std::collections::HashMap;

pub struct LazilyLoadedModels {
    loaded_models: HashMap<ModelId, Model<Texture2D>>,
}

impl LazilyLoadedModels {
    pub fn get_rect_for_object(&mut self, object_data: &ObjectRenderData, pos: Vec2) -> Rect {
        let model = self.lookup_model(&object_data.model_id);
        model_render_rect(model, pos, &object_data.properties)
    }
}

impl ModelAccess<Texture2D> for LazilyLoadedModels {
    fn lookup_model(&mut self, model_id: &ModelId) -> &Model<Texture2D> {
        if !self.loaded_models.contains_key(model_id) {
            load_and_insert_or_default(model_id, &mut self.loaded_models);
        }
        self.loaded_models.get(model_id).unwrap()
    }
}

pub fn draw_model(
    model: &Model<Texture2D>,
    pos: Vec2,
    use_wield_offset: bool,
    properties: &RenderProperties,
    aftik_color_data: &AftikColorData,
) {
    let mut pos = pos;
    if use_wield_offset {
        pos.y += model.wield_offset.y;
        pos.x += f32::from(properties.direction) * model.wield_offset.x;
    }
    let flip_x = model.fixed_orientation && properties.direction == Direction::Left;
    for layer in &model.layers {
        draw_layer(layer, pos, flip_x, properties, aftik_color_data);
    }
}

fn draw_layer(
    layer: &TextureLayer<Texture2D>,
    pos: Vec2,
    flip_x: bool,
    properties: &RenderProperties,
    aftik_color_data: &AftikColorData,
) {
    if !layer.conditions.meets_conditions(properties) {
        return;
    }

    let render_rect = layer_render_rect(layer, pos, if flip_x { -1. } else { 1. });

    for (color_source, texture) in layer.texture.iter() {
        let color = color_source.get_color(aftik_color_data);
        texture::draw_texture_ex(
            texture,
            render_rect.x,
            render_rect.y,
            Color::from_rgba(color.r, color.g, color.b, 255),
            DrawTextureParams {
                dest_size: Some(render_rect.size()),
                flip_x,
                ..Default::default()
            },
        );
    }
}

pub fn model_render_rect(
    model: &Model<Texture2D>,
    pos: Vec2,
    properties: &RenderProperties,
) -> Rect {
    let direction_mod = if model.fixed_orientation {
        1.
    } else {
        properties.direction.into()
    };
    model
        .layers
        .iter()
        .filter(|&layer| layer.conditions.meets_conditions(properties))
        .fold(Rect::new(pos.x, pos.y, 0., 0.), |rect, layer| {
            rect.combine_with(layer_render_rect(layer, pos, direction_mod))
        })
}

fn layer_render_rect(layer: &TextureLayer<Texture2D>, pos: Vec2, direction_mod: f32) -> Rect {
    let dest_size = layer
        .positioning
        .size
        .map(|(width, height)| Vec2::new(f32::from(width), f32::from(height)))
        .unwrap_or_else(|| layer.primary_texture().size());
    Rect::new(
        (pos.x - dest_size.x / 2. + direction_mod * layer.positioning.offset.0.x).floor(),
        pos.y - dest_size.y + layer.positioning.offset.0.y,
        dest_size.x,
        dest_size.y,
    )
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
    models: &mut HashMap<ModelId, Model<Texture2D>>,
) -> Result<(), Error> {
    let model = load_model(&model_id)?;
    models.insert(model_id, model);
    Ok(())
}

fn load_and_insert_or_default(model_id: &ModelId, models: &mut HashMap<ModelId, Model<Texture2D>>) {
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

pub fn load_model(model_id: &ModelId) -> Result<Model<Texture2D>, Error> {
    Ok(model::load_raw_model_from_path(model_id.file_path())?.load(&mut super::InPlaceLoader)?)
}
