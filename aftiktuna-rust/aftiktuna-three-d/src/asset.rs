use crate::{BuiltinFonts, Rect};
use aftiktuna::asset::background::{self as background_base, BGData};
use aftiktuna::asset::color::AftikColorData;
use aftiktuna::asset::model::{self, Model, TextureLayer};
use aftiktuna::asset::{self as asset_base, TextureLoader};
use aftiktuna::core::area::BackgroundId;
use aftiktuna::core::display::{AftikColorId, ModelId};
use aftiktuna::view::area::{ObjectRenderData, RenderProperties};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

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

pub struct Assets {
    pub backgrounds: BackgroundMap,
    pub models: LazilyLoadedModels,
    pub aftik_colors: HashMap<AftikColorId, AftikColorData>,
    pub left_mouse_icon: three_d::Texture2DRef,
    pub side_arrow_texture: three_d::Texture2DRef,
    pub builtin_fonts: Rc<BuiltinFonts>,
}

impl Assets {
    pub fn load(context: three_d::Context, builtin_fonts: Rc<BuiltinFonts>) -> Result<Self, Error> {
        let left_mouse_icon = load_texture("left_mouse", &context)?;
        let side_arrow_texture = load_texture("side_arrow", &context)?;
        Ok(Self {
            backgrounds: BackgroundMap::load(context.clone())?,
            models: LazilyLoadedModels::new(context)?,
            aftik_colors: asset_base::color::load_aftik_color_data()?,
            left_mouse_icon,
            side_arrow_texture,
            builtin_fonts,
        })
    }
}

struct CachedLoader(HashMap<String, three_d::Texture2DRef>, three_d::Context);

impl CachedLoader {
    fn new(context: three_d::Context) -> Self {
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

fn load_texture(
    name: &str,
    context: &three_d::Context,
) -> Result<three_d::Texture2DRef, three_d_asset::Error> {
    let path = format!("assets/texture/{name}.png");

    let texture: three_d::CpuTexture = three_d_asset::io::load_and_deserialize(path)?;
    Ok(three_d::Texture2DRef::from_cpu_texture(context, &texture))
}

pub struct BackgroundMap(HashMap<BackgroundId, BGData<three_d::Texture2DRef>>);

impl BackgroundMap {
    fn load(context: three_d::Context) -> Result<Self, Error> {
        let mut texture_loader = CachedLoader::new(context);
        let background_data = background_base::load_raw_backgrounds()?;
        if !background_data.contains_key(&BackgroundId::blank()) {
            return Err(Error::MissingBlankBackground);
        }

        Ok(Self(
            background_data
                .into_iter()
                .map(|(id, data)| {
                    data.load(&mut texture_loader)
                        .map(|loaded_data| (id, loaded_data))
                })
                .collect::<Result<_, _>>()?,
        ))
    }

    pub fn get_or_default<'a>(&'a self, id: &BackgroundId) -> &'a BGData<three_d::Texture2DRef> {
        self.0
            .get(id)
            .or_else(|| self.0.get(&BackgroundId::blank()))
            .expect("Missing blank texture")
    }
}

pub struct LazilyLoadedModels {
    texture_loader: CachedLoader,
    loaded_models: HashMap<ModelId, Model<three_d::Texture2DRef>>,
}

impl LazilyLoadedModels {
    fn new(context: three_d::Context) -> Result<Self, Error> {
        let mut models = Self {
            texture_loader: CachedLoader::new(context),
            loaded_models: HashMap::new(),
        };
        models.load_and_insert_model(&ModelId::unknown())?;
        models.load_and_insert_model(&ModelId::small_unknown())?;
        Ok(models)
    }

    pub fn lookup_model(&mut self, model_id: &ModelId) -> &Model<three_d::Texture2DRef> {
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

fn model_render_rect(
    model: &Model<three_d::Texture2DRef>,
    pos: three_d::Vec2,
    properties: &RenderProperties,
) -> Rect {
    model
        .layers
        .iter()
        .filter(|&layer| layer.conditions.meets_conditions(properties))
        .fold(Rect::new(pos.x, pos.y, 0., 0.), |rect, layer| {
            rect.combine(layer_render_rect(layer, pos))
        })
}

fn layer_render_rect(layer: &TextureLayer<three_d::Texture2DRef>, pos: three_d::Vec2) -> Rect {
    let (width, height) = layer
        .positioning
        .size
        .map(|(width, height)| (f32::from(width), f32::from(height)))
        .unwrap_or_else(|| (layer.texture.width() as f32, layer.texture.height() as f32));
    Rect::new(
        pos.x - width / 2.,
        pos.y - f32::from(layer.positioning.y_offset),
        width,
        height,
    )
}
