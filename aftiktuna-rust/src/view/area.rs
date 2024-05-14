use crate::command::suggestion;
use crate::command::suggestion::InteractionType;
use crate::core::area::{Area, BackgroundType};
use crate::core::inventory::Held;
use crate::core::item::CanWield;
use crate::core::position::{Coord, Direction, Pos};
use crate::core::status::Health;
use crate::core::{inventory, BlockType, Door, GameState, IsCut};
use crate::deref_clone;
use crate::view::name::NameData;
use crate::view::{capitalize, Messages};
use hecs::{Entity, EntityRef, World};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::ops::Deref;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TextureType(String);

impl TextureType {
    pub fn unknown() -> Self {
        Self::new("unknown")
    }
    pub fn small_unknown() -> Self {
        Self::new("small_unknown")
    }
    pub fn aftik() -> Self {
        Self::creature("aftik")
    }

    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn item(name: &str) -> Self {
        Self(format!("item/{name}"))
    }

    pub fn creature(name: &str) -> Self {
        Self(format!("creature/{name}"))
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

impl Default for TextureType {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Symbol(pub char);

impl Symbol {
    pub fn from_name(name: &str) -> Self {
        Self(name.chars().next().unwrap())
    }
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum OrderWeight {
    Item,
    Controlled,
    #[default]
    Creature,
    Background,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AftikColor {
    #[default]
    Mint,
    Cerulean,
    Plum,
    Green,
}

pub(super) fn area_view_messages(render_data: &RenderData) -> Messages {
    let mut messages = Messages::default();
    messages.add(format!("{}:", render_data.area_label));
    print_area(&mut messages.0, render_data);
    messages
}

fn print_area(lines: &mut Vec<String>, render_data: &RenderData) {
    let area_size = render_data.area_size;
    let mut symbols_by_pos = init_symbol_vectors(area_size);
    let mut labels = Vec::new();

    for object in &render_data.objects {
        symbols_by_pos[object.coord].push((object.symbol.0, object.weight));

        let label = format!("{}: {}", object.symbol.0, object.modified_name,);
        if !labels.contains(&label) {
            labels.push(label);
        }
    }

    for symbol_column in &mut symbols_by_pos {
        symbol_column.sort_by(|a, b| b.1.cmp(&a.1));
    }

    let rows: usize = max(1, symbols_by_pos.iter().map(Vec::len).max().unwrap());
    for row in (0..rows).rev() {
        let base_symbol = if row == 0 { '_' } else { ' ' };
        let mut symbols = vec![base_symbol; area_size];
        for pos in 0..area_size {
            if let Some(&(symbol, _)) = symbols_by_pos[pos].get(row) {
                symbols[pos] = symbol;
            }
        }
        lines.push(symbols.iter().collect::<String>());
    }
    for labels in labels.chunks(3) {
        lines.push(labels.join("   "));
    }
}

fn init_symbol_vectors<T>(size: usize) -> Vec<Vec<T>> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push(Vec::new());
    }
    symbols
}

fn get_name(world: &World, entity: Entity, name: String) -> String {
    if let Ok(door_pair) = world.get::<&Door>(entity).map(|door| door.door_pair) {
        if let Ok(blocking) = world.get::<&BlockType>(door_pair) {
            return format!("{} ({})", name, blocking.description());
        }
    }
    name
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderData {
    pub area_label: String,
    pub area_size: Coord,
    pub background: BackgroundType,
    pub background_offset: Option<Coord>,
    pub character_coord: Coord,
    pub inventory: Vec<ItemProfile>,
    pub objects: Vec<ObjectRenderData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRenderData {
    pub coord: Coord,
    pub weight: OrderWeight,
    pub texture_type: TextureType,
    pub modified_name: String,
    pub name: String,
    pub symbol: Symbol,
    pub wielded_item: Option<TextureType>,
    pub interactions: Vec<InteractionType>,
    #[serde(flatten)]
    pub properties: RenderProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderProperties {
    pub direction: Direction,
    pub aftik_color: Option<AftikColor>,
    pub is_cut: bool,
    pub is_alive: bool,
}

impl Default for RenderProperties {
    fn default() -> Self {
        Self {
            direction: Direction::Right,
            aftik_color: None,
            is_cut: false,
            is_alive: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemProfile {
    pub name: String,
    pub is_wieldable: bool,
    pub is_wielded: bool,
}

impl ItemProfile {
    fn create(item: EntityRef) -> Self {
        Self {
            name: NameData::find_for_ref(item).base().to_string(),
            is_wieldable: item.satisfies::<&CanWield>(),
            is_wielded: item.get::<&Held>().map_or(false, |held| held.is_in_hand()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub(super) fn prepare_render_data(state: &GameState) -> RenderData {
    let character_pos = state.world.get::<&Pos>(state.controlled).unwrap();
    let area = state.world.get::<&Area>(character_pos.get_area()).unwrap();

    let mut objects: Vec<ObjectRenderData> = state
        .world
        .query::<(&Pos, &Symbol)>()
        .iter()
        .filter(|&(_, (pos, _))| pos.is_in(character_pos.get_area()))
        .map(|(entity, (pos, &symbol))| build_object_data(state, entity, pos, symbol))
        .collect();
    objects.sort_by(|data1, data2| data2.weight.cmp(&data1.weight));

    let inventory = inventory::get_held(&state.world, state.controlled)
        .into_iter()
        .map(|item| ItemProfile::create(state.world.entity(item).unwrap()))
        .collect();
    RenderData {
        area_label: area.label.clone(),
        area_size: area.size,
        background: area.background.clone(),
        background_offset: area.background_offset,
        character_coord: character_pos.get_coord(),
        inventory,
        objects,
    }
}

fn build_object_data(
    state: &GameState,
    entity: Entity,
    pos: &Pos,
    symbol: Symbol,
) -> ObjectRenderData {
    let entity_ref = state.world.entity(entity).unwrap();
    let name_data = NameData::find_for_ref(entity_ref);
    let properties = RenderProperties {
        direction: entity_ref
            .get::<&Direction>()
            .map(deref_clone)
            .unwrap_or_default(),
        aftik_color: entity_ref.get::<&AftikColor>().map(deref_clone),
        is_cut: entity_ref.satisfies::<&IsCut>(),
        is_alive: entity_ref
            .get::<&Health>()
            .map_or(true, |health| health.is_alive()),
    };
    ObjectRenderData {
        coord: pos.get_coord(),
        weight: entity_ref
            .get::<&OrderWeight>()
            .map(deref_clone)
            .unwrap_or_default(),
        texture_type: entity_ref
            .get::<&TextureType>()
            .map(deref_clone)
            .unwrap_or_default(),
        modified_name: get_name(&state.world, entity, capitalize(name_data.base())),
        name: capitalize(name_data.base()),
        symbol,
        wielded_item: find_wielded_item_texture(&state.world, entity),
        interactions: suggestion::interactions_for(entity, state),
        properties,
    }
}

fn find_wielded_item_texture(world: &World, holder: Entity) -> Option<TextureType> {
    let item = inventory::get_wielded(world, holder)?;
    world
        .get::<&TextureType>(item)
        .ok()
        .map(|texture_type| texture_type.deref().clone())
}
