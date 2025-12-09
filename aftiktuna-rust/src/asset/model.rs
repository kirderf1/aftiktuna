use super::TextureLoader;
use super::color::ColorSource;
use crate::core::display::{CreatureVariant, DialogueExpression, ModelId};
use crate::view::area::ObjectProperties;
use crate::{OneOrList, Range, Vec2};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Serializer};
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Model<T> {
    pub layers: Vec<TextureLayer<T>>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub wield_offset: Vec2,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub z_offset: i16,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub order_weight: i16,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub fixed_orientation: bool,
    #[serde(default = "value_true", skip_serializing_if = "is_true")]
    pub has_x_displacement: bool,
    #[serde(
        default = "y_displacement_default",
        skip_serializing_if = "is_y_displacement_default"
    )]
    pub z_displacement: i16,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub large_displacement: bool,
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
            order_weight: self.order_weight,
            fixed_orientation: self.fixed_orientation,
            has_x_displacement: self.has_x_displacement,
            z_displacement: self.z_displacement,
            large_displacement: self.large_displacement,
            group_placement: self.group_placement.clone(),
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureLayer<T> {
    #[serde(default, skip_serializing_if = "LayerConditionList::is_empty")]
    pub condition: LayerConditionList,
    #[serde(flatten)]
    pub positioning: LayerPositioning,
    #[serde(flatten)]
    pub textures_or_children: TexturesOrChildren<T>,
}

impl TextureLayer<String> {
    fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<TextureLayer<T>, E> {
        let textures_or_children = self.textures_or_children.load(loader)?;
        Ok(TextureLayer {
            textures_or_children,
            positioning: self.positioning.clone(),
            condition: self.condition.clone(),
        })
    }
}

pub fn texture_path(texture: &str) -> String {
    format!("object/{texture}")
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TexturesOrChildren<T> {
    Texture(ColoredTextures<T>),
    Children(Vec<TextureLayer<T>>),
}

impl TexturesOrChildren<String> {
    fn load<T, E>(
        &self,
        loader: &mut impl TextureLoader<T, E>,
    ) -> Result<TexturesOrChildren<T>, E> {
        Ok(match self {
            TexturesOrChildren::Texture(colored_textures) => {
                let mut colored_textures = colored_textures.load(loader)?;
                colored_textures.0.reverse();
                TexturesOrChildren::Texture(colored_textures)
            }
            TexturesOrChildren::Children(texture_layers) => {
                let mut layer_list: Vec<TextureLayer<T>> = texture_layers
                    .iter()
                    .map(|layer| layer.load(loader))
                    .collect::<Result<_, E>>()?;
                layer_list.reverse();
                TexturesOrChildren::Children(layer_list)
            }
        })
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "TextureOrMap<T>")]
pub struct ColoredTextures<T>(Vec<(ColorSource, T)>);

impl<T> ColoredTextures<T> {
    pub fn primary_texture(&self) -> &T {
        &self.0.first().unwrap().1
    }

    pub fn iter(&self) -> impl Iterator<Item = (ColorSource, &T)> {
        self.0.iter().map(|(color, texture)| (*color, texture))
    }
}

impl ColoredTextures<String> {
    fn load<T, E>(&self, loader: &mut impl TextureLoader<T, E>) -> Result<ColoredTextures<T>, E> {
        Ok(ColoredTextures(
            self.0
                .iter()
                .map(|(color, texture)| Ok((*color, loader.load_texture(texture_path(texture))?)))
                .collect::<Result<_, E>>()?,
        ))
    }
}

impl<T> From<T> for ColoredTextures<T> {
    fn from(value: T) -> Self {
        Self(vec![(ColorSource::Uncolored, value)])
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TextureOrMap<T> {
    Texture(T),
    List(Vec<(ColorSource, T)>),
}

impl<T> TryFrom<TextureOrMap<T>> for ColoredTextures<T> {
    type Error = &'static str;

    fn try_from(value: TextureOrMap<T>) -> Result<Self, Self::Error> {
        match value {
            TextureOrMap::Texture(texture) => Ok(Self::from(texture)),
            TextureOrMap::List(list) => {
                if list.is_empty() {
                    Err("Texture list must not be empty")
                } else {
                    Ok(Self(list))
                }
            }
        }
    }
}

impl<T: Serialize> Serialize for ColoredTextures<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.len() == 1
            && let Some((ColorSource::Uncolored, texture)) = self.0.first()
        {
            texture.serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayerPositioning {
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub size: Option<(i16, i16)>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub offset: Range<Vec2>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub anchor: Vec2,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub rotation: Range<f32>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub animation_length: f32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(from = "OneOrList<LayerCondition>", into = "OneOrList<LayerCondition>")]
pub struct LayerConditionList(pub Vec<LayerCondition>);

impl LayerConditionList {
    pub fn test(&self, properties: &ObjectProperties) -> bool {
        self.0.iter().all(|condition| condition.test(properties))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<OneOrList<LayerCondition>> for LayerConditionList {
    fn from(value: OneOrList<LayerCondition>) -> Self {
        match value {
            OneOrList::One(condition) => Self(vec![condition]),
            OneOrList::List(list) => Self(list),
        }
    }
}

impl From<LayerConditionList> for OneOrList<LayerCondition> {
    fn from(value: LayerConditionList) -> Self {
        if value.0.len() == 1 {
            Self::One(value.0.into_iter().next().unwrap())
        } else {
            Self::List(value.0)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerCondition {
    Cut(bool),
    Alive(bool),
    Hurt(bool),
    Expression(DialogueExpression),
    Variant(CreatureVariant),
}

impl LayerCondition {
    fn test(&self, properties: &ObjectProperties) -> bool {
        match self {
            LayerCondition::Cut(if_cut) => *if_cut == properties.is_cut,
            LayerCondition::Alive(if_alive) => *if_alive == properties.is_alive,
            LayerCondition::Hurt(if_hurt) => *if_hurt == properties.is_badly_hurt,
            LayerCondition::Expression(dialogue_expression) => {
                *dialogue_expression == properties.expression
            }
            LayerCondition::Variant(creature_variant) => {
                properties.creature_variant_set.0.contains(creature_variant)
            }
        }
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
    super::load_from_json(file_path)
}

pub trait ModelAccess<T> {
    fn lookup_model(&mut self, model_id: &ModelId) -> &Model<T>;
}
