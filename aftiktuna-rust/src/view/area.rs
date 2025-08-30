use crate::asset::NounDataMap;
use crate::asset::background::ParallaxLayer;
use crate::command::suggestion;
use crate::command::suggestion::InteractionType;
use crate::core::area::{Area, BackgroundId};
use crate::core::display::{AftikColorId, ModelId};
use crate::core::inventory::Held;
use crate::core::item::{CanWield, ItemType};
use crate::core::name::{NameData, NameWithAttribute};
use crate::core::position::{Coord, Direction, Pos};
use crate::core::status::{self, Health};
use crate::core::{BlockType, Door, IsCut, inventory};
use crate::deref_clone;
use crate::game_loop::GameState;
use crate::view::text;
use hecs::{Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderData {
    pub area_size: Coord,
    pub background: BackgroundId,
    pub background_offset: i32,
    pub extra_background_layers: Vec<ParallaxLayer<String>>,
    pub darkness: f32,
    pub character_coord: Coord,
    pub inventory: Vec<ItemProfile>,
    pub objects: Vec<ObjectRenderData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRenderData {
    pub coord: Coord,
    pub model_id: ModelId,
    pub hash: u64,
    pub is_controlled: bool,
    pub name_data: Option<ObjectNameData>,
    pub wielded_item: Option<ModelId>,
    pub interactions: Vec<InteractionType>,
    #[serde(flatten)]
    pub properties: ObjectProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNameData {
    pub modified_name: String,
    pub name: String,
}

impl ObjectNameData {
    fn build(entity_ref: EntityRef, world: &World, noun_map: &NounDataMap) -> Option<Self> {
        let name = NameWithAttribute::lookup_option_by_ref(entity_ref, noun_map)?.base();
        Some(Self {
            modified_name: text::capitalize(get_extended_name(&name, entity_ref, world)),
            name: text::capitalize(&name),
        })
    }
}

fn get_extended_name(name: &str, entity_ref: EntityRef, world: &World) -> String {
    if let Some(door) = entity_ref.get::<&Door>()
        && let Ok(blocking) = world.get::<&BlockType>(door.door_pair)
    {
        return format!("{name} ({})", blocking.description());
    }

    if !status::is_alive_ref(entity_ref) {
        return format!("Corpse of {name}");
    }

    if entity_ref.satisfies::<&status::IsStunned>() {
        return format!("{name} (stunned)");
    }

    name.to_owned()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectProperties {
    pub direction: Direction,
    pub aftik_color: Option<AftikColorId>,
    pub is_cut: bool,
    pub is_alive: bool,
    pub is_badly_hurt: bool,
}

impl Default for ObjectProperties {
    fn default() -> Self {
        Self {
            direction: Direction::Right,
            aftik_color: None,
            is_cut: false,
            is_alive: true,
            is_badly_hurt: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemProfile {
    pub name: String,
    pub is_wieldable: bool,
    pub is_wielded: bool,
    #[serde(default)]
    pub is_usable: bool,
}

impl ItemProfile {
    fn create(item: EntityRef, noun_map: &NounDataMap) -> Self {
        Self {
            name: NameData::find_by_ref(item, noun_map).base(),
            is_wieldable: item.satisfies::<&CanWield>(),
            is_wielded: item.get::<&Held>().is_some_and(|held| held.is_in_hand()),
            is_usable: item
                .get::<&ItemType>()
                .is_some_and(|item_type| item_type.is_usable()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub(super) fn prepare_render_data(state: &GameState, noun_map: &NounDataMap) -> RenderData {
    let character_pos = state.world.get::<&Pos>(state.controlled).unwrap();
    let area = state.world.get::<&Area>(character_pos.get_area()).unwrap();

    let objects: Vec<ObjectRenderData> = state
        .world
        .query::<&Pos>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(character_pos.get_area()))
        .map(|(entity, pos)| build_object_data(state, entity, pos, noun_map))
        .collect();

    let inventory = inventory::get_held(&state.world, state.controlled)
        .into_iter()
        .map(|item| ItemProfile::create(state.world.entity(item).unwrap(), noun_map))
        .collect();
    RenderData {
        area_size: area.size,
        background: area.background.clone(),
        background_offset: area.background_offset,
        extra_background_layers: area.extra_background_layers.clone(),
        darkness: area.darkness,
        character_coord: character_pos.get_coord(),
        inventory,
        objects,
    }
}

fn build_object_data(
    state: &GameState,
    entity: Entity,
    pos: &Pos,
    noun_map: &NounDataMap,
) -> ObjectRenderData {
    let entity_ref = state.world.entity(entity).unwrap();
    let properties = ObjectProperties {
        direction: entity_ref
            .get::<&Direction>()
            .map(deref_clone)
            .unwrap_or_default(),
        aftik_color: entity_ref.get::<&AftikColorId>().map(deref_clone),
        is_cut: entity_ref.satisfies::<&IsCut>(),
        is_alive: entity_ref
            .get::<&Health>()
            .is_none_or(|health| health.is_alive()),
        is_badly_hurt: entity_ref
            .get::<&Health>()
            .is_some_and(|health| health.is_badly_hurt()),
    };
    ObjectRenderData {
        coord: pos.get_coord(),
        model_id: entity_ref
            .get::<&ModelId>()
            .map(deref_clone)
            .unwrap_or_default(),
        hash: {
            use std::hash::{DefaultHasher, Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            entity.hash(&mut hasher);
            hasher.finish()
        },
        is_controlled: entity == state.controlled,
        name_data: ObjectNameData::build(entity_ref, &state.world, noun_map),
        wielded_item: find_wielded_item_texture(&state.world, entity),
        interactions: suggestion::interactions_for(entity, state),
        properties,
    }
}

fn find_wielded_item_texture(world: &World, holder: Entity) -> Option<ModelId> {
    let item = inventory::get_wielded(world, holder)?;
    world
        .get::<&ModelId>(item)
        .ok()
        .map(|texture_type| texture_type.deref().clone())
}
