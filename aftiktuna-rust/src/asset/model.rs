use super::TextureLoader;
use super::color::ColorSource;
use crate::core::display::ModelId;
use crate::view::area::RenderProperties;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Model<T> {
    pub layers: Vec<TextureLayer<T>>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub wield_offset: (i16, i16),
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub z_offset: i16,
    #[serde(default = "value_true", skip_serializing_if = "is_true")]
    pub has_x_displacement: bool,
    #[serde(
        default = "y_displacement_default",
        skip_serializing_if = "is_y_displacement_default"
    )]
    pub z_displacement: i16,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub group_placement: GroupPlacement,
}

fn value_true() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

fn y_displacement_default() -> i16 {
    15
}

fn is_y_displacement_default(y_displacement: &i16) -> bool {
    *y_displacement == y_displacement_default()
}

impl Model<String> {
    pub fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<Model<T>, E> {
        let mut layers = Vec::new();
        for layer in &self.layers {
            layers.push(layer.load(loader)?);
        }
        layers.reverse();
        Ok(Model {
            layers,
            wield_offset: self.wield_offset,
            z_offset: self.z_offset,
            has_x_displacement: self.has_x_displacement,
            z_displacement: self.z_displacement,
            group_placement: self.group_placement.clone(),
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
    pub fn meets_conditions(&self, properties: &RenderProperties) -> bool {
        (self.if_cut.is_none() || self.if_cut == Some(properties.is_cut))
            && (self.if_alive.is_none() || self.if_alive == Some(properties.is_alive))
            && (self.if_hurt.is_none() || self.if_hurt == Some(properties.is_badly_hurt))
    }
}

pub type Offsets = Vec<(i16, i16)>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "IndexMap<u16, Offsets>")]
pub struct GroupPlacement(IndexMap<u16, Offsets>);

impl GroupPlacement {
    pub fn position(&self, count: u16) -> Vec<Offsets> {
        let mut groups = Vec::new();
        let mut remaining_count = count;
        while remaining_count > 0 {
            let consumed_count = self.floor_index(remaining_count);
            groups.push(self.0[&consumed_count].clone());
            remaining_count -= consumed_count;
        }
        groups
    }

    fn floor_index(&self, count: u16) -> u16 {
        self.0
            .keys()
            .copied()
            .filter(|&i| i <= count)
            .max()
            .unwrap()
    }
}

impl Default for GroupPlacement {
    fn default() -> Self {
        let mut map = IndexMap::new();
        map.insert(1, vec![(0, 0)]);
        Self(map)
    }
}

impl TryFrom<IndexMap<u16, Offsets>> for GroupPlacement {
    type Error = String;

    fn try_from(value: IndexMap<u16, Offsets>) -> Result<Self, Self::Error> {
        if value.contains_key(&0) {
            return Err("May not contain position group 0".to_string());
        }
        if !value.contains_key(&1) {
            return Err("Must contain at least position group 1".to_string());
        }
        for (key, positions) in value.iter() {
            if positions.len() != usize::from(*key) {
                return Err(format!(
                    "Position group {key} has wrong number of positions: {}",
                    positions.len()
                ));
            }
        }
        Ok(Self(value))
    }
}

pub fn load_raw_model_from_path(
    file_path: impl AsRef<Path>,
) -> Result<Model<String>, super::Error> {
    let file = File::open(file_path)?;
    Ok(serde_json::from_reader::<_, Model<String>>(file)?)
}

pub trait ModelAccess<T> {
    fn lookup_model(&mut self, model_id: &ModelId) -> &Model<T>;
}
