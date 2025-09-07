use crate::Rect;
use aftiktuna::asset::background::{self, BGData, ParallaxLayer};
use aftiktuna::asset::model::{
    self, ColoredTextures, LayerPositioning, Model, ModelAccess, TextureLayer,
};
use aftiktuna::asset::{self as asset_base, TextureLoader};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::ModelId;
use aftiktuna::view::area::{ObjectProperties, ObjectRenderData};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Asset(asset_base::Error),
    ThreeD(three_d_asset::Error),
    MissingBlankBackground,
}

impl From<asset_base::Error> for Error {
    fn from(value: asset_base::Error) -> Self {
        Self::Asset(value)
    }
}

impl From<three_d_asset::Error> for Error {
    fn from(value: three_d_asset::Error) -> Self {
        Self::ThreeD(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Asset(error) => Display::fmt(error, f),
            Error::ThreeD(error) => Display::fmt(error, f),
            Error::MissingBlankBackground => {
                Display::fmt("Missing Background: Blank background texture must exist", f)
            }
        }
    }
}

pub struct CachedLoader(HashMap<String, three_d::Texture2DRef>, three_d::Context);

impl CachedLoader {
    pub fn new(context: three_d::Context) -> Self {
        Self(HashMap::new(), context)
    }
}

impl TextureLoader<three_d::Texture2DRef, three_d_asset::Error> for CachedLoader {
    fn load_texture(
        &mut self,
        name: String,
    ) -> Result<three_d::Texture2DRef, three_d_asset::Error> {
        if let Some(texture) = self.0.get(&name) {
            return Ok(texture.clone());
        }

        let texture = load_texture(&name, &self.1)?;
        self.0.insert(name, texture.clone());
        Ok(texture)
    }
}

pub fn load_texture(
    name: &str,
    context: &three_d::Context,
) -> Result<three_d::Texture2DRef, three_d_asset::Error> {
    let path = format!("assets/texture/{name}.png");

    let mut texture: three_d::CpuTexture = three_d_asset::io::load_and_deserialize(path)?;
    texture.wrap_s = three_d::Wrapping::ClampToEdge;
    texture.wrap_t = three_d::Wrapping::ClampToEdge;
    Ok(three_d::Texture2DRef::from_cpu_texture(context, &texture))
}

pub struct BackgroundMap(
    CachedLoader,
    HashMap<BackgroundId, BGData<three_d::Texture2DRef>>,
);

impl BackgroundMap {
    pub fn load(context: three_d::Context) -> Result<Self, Error> {
        let background_data = background::load_raw_backgrounds()?;
        if !background_data.contains_key(&BackgroundId::blank()) {
            return Err(Error::MissingBlankBackground);
        }

        let mut texture_loader = CachedLoader::new(context);
        let backgrounds_map = background_data
            .into_iter()
            .map(|(id, data)| {
                data.load(&mut texture_loader)
                    .map(|loaded_data| (id, loaded_data))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self(texture_loader, backgrounds_map))
    }

    pub fn get_or_default<'a>(&'a self, id: &BackgroundId) -> &'a BGData<three_d::Texture2DRef> {
        self.1
            .get(id)
            .or_else(|| self.1.get(&BackgroundId::blank()))
            .expect("Missing blank texture")
    }

    pub fn load_extra_layers(
        &mut self,
        extra_layers: &[ParallaxLayer<String>],
    ) -> Result<Vec<ParallaxLayer<three_d::Texture2DRef>>, three_d_asset::Error> {
        extra_layers
            .iter()
            .map(|layer| layer.load(&mut self.0))
            .collect()
    }
}

pub struct LazilyLoadedModels {
    texture_loader: CachedLoader,
    loaded_models: HashMap<ModelId, Model<three_d::Texture2DRef>>,
}

impl LazilyLoadedModels {
    pub fn new(context: three_d::Context) -> Result<Self, Error> {
        let mut models = Self {
            texture_loader: CachedLoader::new(context),
            loaded_models: HashMap::new(),
        };
        models.load_and_insert_model(&ModelId::unknown())?;
        models.load_and_insert_model(&ModelId::small_unknown())?;
        Ok(models)
    }

    fn load_and_insert_model(&mut self, model_id: &ModelId) -> Result<(), Error> {
        let model = model::load_raw_model_from_path(model_id.file_path())?
            .load(&mut self.texture_loader)?;
        self.loaded_models.insert(model_id.clone(), model);
        Ok(())
    }

    pub fn get_rect_for_object(
        &mut self,
        object_data: &ObjectRenderData,
        pos: three_d::Vec2,
    ) -> Rect {
        let model = self.lookup_model(&object_data.model_id);
        model_render_rect(model, pos, &object_data.properties)
    }
}

impl ModelAccess<three_d::Texture2DRef> for LazilyLoadedModels {
    fn lookup_model(&mut self, model_id: &ModelId) -> &Model<three_d::Texture2DRef> {
        if !self.loaded_models.contains_key(model_id) {
            if let Err(error) = self.load_and_insert_model(model_id) {
                let path = model_id.path();
                eprintln!("Unable to load model \"{path}\": {error}");
                let fallback_id = if path.starts_with("item/") {
                    ModelId::small_unknown()
                } else {
                    ModelId::unknown()
                };
                let fallback_model = self.loaded_models.get(&fallback_id).unwrap().clone();
                self.loaded_models.insert(model_id.clone(), fallback_model);
            }
        }
        self.loaded_models.get(model_id).unwrap()
    }
}

fn model_render_rect(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &ObjectProperties,
) -> Rect {
    let direction_mod = if model.fixed_orientation {
        1.
    } else {
        properties.direction.into()
    };
    layer_list_render_rect(&model.layers, pos, direction_mod, properties)
}

fn layer_list_render_rect(
    layer_list: &[TextureLayer<three_d::Texture2DRef>],
    pos: three_d::Vector2<f32>,
    direction_mod: f32,
    properties: &ObjectProperties,
) -> Rect {
    layer_list
        .iter()
        .filter(|&layer| layer.conditions.meets_conditions(properties))
        .fold(Rect::new(pos.x, pos.y, 0., 0.), |rect, layer| {
            rect.combine(layer_render_rect(layer, pos, direction_mod, properties))
        })
}

fn layer_render_rect(
    layer: &TextureLayer<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    direction_mod: f32,
    properties: &ObjectProperties,
) -> Rect {
    match &layer.textures_or_children {
        model::TexturesOrChildren::Texture(textures) => {
            textures_render_rect(textures, pos, &layer.positioning, direction_mod)
        }
        model::TexturesOrChildren::Children(texture_layers) => {
            let layer_pos = pos + crate::to_vec(layer.positioning.offset.0, direction_mod);
            layer_list_render_rect(texture_layers, layer_pos, direction_mod, properties)
        }
    }
}

fn textures_render_rect(
    textures: &ColoredTextures<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    positioning: &LayerPositioning,
    direction_mod: f32,
) -> Rect {
    let (width, height) = positioning
        .size
        .map(|(width, height)| (f32::from(width), f32::from(height)))
        .unwrap_or_else(|| {
            (
                textures.primary_texture().width() as f32,
                textures.primary_texture().height() as f32,
            )
        });
    let layer_pos = pos + crate::to_vec(positioning.offset.0, direction_mod);
    Rect::new(layer_pos.x - width / 2., layer_pos.y, width, height)
}
