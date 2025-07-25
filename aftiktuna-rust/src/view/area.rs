use crate::asset::background::ParallaxLayer;
use crate::command::suggestion;
use crate::command::suggestion::InteractionType;
use crate::core::area::{Area, BackgroundId};
use crate::core::display::{AftikColorId, ModelId, OrderWeight, Symbol};
use crate::core::inventory::Held;
use crate::core::item::{CanWield, Medkit, Usable};
use crate::core::name::{NameData, NameWithAttribute};
use crate::core::position::{Coord, Direction, Pos};
use crate::core::status::{self, Health};
use crate::core::{BlockType, Door, IsCut, inventory};
use crate::deref_clone;
use crate::game_loop::GameState;
use crate::view::text;
use hecs::{Entity, EntityRef, Or, World};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::ops::Deref;

pub(super) fn push_area_view_lines(text_lines: &mut Vec<String>, render_data: &RenderData) {
    text_lines.push(format!("{name}:", name = render_data.area_label));

    let area_size = render_data.area_size;
    let mut symbols_by_pos: Vec<Vec<(Symbol, OrderWeight)>> =
        init_symbol_vectors(area_size as usize);
    let mut labels: HashMap<Symbol, String> = HashMap::new();

    for object in &render_data.objects {
        if let Some(name_data) = &object.name_data {
            let symbol = insert_label_at_available_symbol(name_data, &mut labels);

            symbols_by_pos[object.coord as usize].push((symbol, object.weight));
        }
    }

    for symbol_column in &mut symbols_by_pos {
        symbol_column.sort_by(|a, b| b.1.cmp(&a.1));
    }

    let rows: usize = max(1, symbols_by_pos.iter().map(Vec::len).max().unwrap());
    for row in (0..rows).rev() {
        let base_symbol = if row == 0 { '_' } else { ' ' };
        let mut symbols = vec![base_symbol; area_size as usize];
        for pos in 0..symbols.len() {
            if let Some(&(symbol, _)) = symbols_by_pos[pos].get(row) {
                symbols[pos] = symbol.0;
            }
        }
        text_lines.push(symbols.iter().collect::<String>());
    }

    let mut labels = labels
        .into_iter()
        .map(|(Symbol(c), label)| format!("{c}: {label}"))
        .collect::<Vec<_>>();
    labels.sort();
    for labels in labels.chunks(3) {
        text_lines.push(labels.join("   "));
    }
}

fn init_symbol_vectors<T>(size: usize) -> Vec<Vec<T>> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push(Vec::new());
    }
    symbols
}

const BACKUP_CHARS: [char; 56] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
];

fn insert_label_at_available_symbol(
    name_data: &ObjectNameData,
    labels: &mut HashMap<Symbol, String>,
) -> Symbol {
    let label = &name_data.modified_name;
    for symbol in [name_data.symbol]
        .into_iter()
        .chain(BACKUP_CHARS.into_iter().map::<Symbol, _>(Symbol))
    {
        let existing_label = labels.get(&symbol);

        if existing_label.is_none() {
            labels.insert(symbol, label.to_owned());
            return symbol;
        }
        let existing_label = existing_label.unwrap();
        if existing_label.eq(label) {
            return symbol;
        }
    }

    eprintln!("Too many unique symbols. Some symbols will overlap.");
    name_data.symbol
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderData {
    pub area_label: String,
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
    pub weight: OrderWeight,
    pub model_id: ModelId,
    #[serde(default)] // backwards-compatibility with 4.0
    pub hash: u64,
    pub name_data: Option<ObjectNameData>,
    pub wielded_item: Option<ModelId>,
    pub interactions: Vec<InteractionType>,
    #[serde(flatten)]
    pub properties: RenderProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNameData {
    pub modified_name: String,
    pub name: String,
    pub symbol: Symbol,
}

impl ObjectNameData {
    fn build(entity_ref: EntityRef, world: &World) -> Option<Self> {
        let name = NameWithAttribute::lookup_option_by_ref(entity_ref)?.base();
        Some(Self {
            modified_name: text::capitalize(get_extended_name(&name, entity_ref, world)),
            name: text::capitalize(&name),
            symbol: entity_ref
                .get::<&Symbol>()
                .map(deref_clone)
                .unwrap_or_else(|| Symbol::from_name(&name)),
        })
    }
}

fn get_extended_name(name: &str, entity_ref: EntityRef, world: &World) -> String {
    if let Some(door) = entity_ref.get::<&Door>() {
        if let Ok(blocking) = world.get::<&BlockType>(door.door_pair) {
            return format!("{name} ({})", blocking.description());
        }
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
pub struct RenderProperties {
    pub direction: Direction,
    pub aftik_color: Option<AftikColorId>,
    pub is_cut: bool,
    pub is_alive: bool,
    pub is_badly_hurt: bool,
}

impl Default for RenderProperties {
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
    fn create(item: EntityRef) -> Self {
        Self {
            name: NameData::find_by_ref(item).base().to_string(),
            is_wieldable: item.satisfies::<&CanWield>(),
            is_wielded: item.get::<&Held>().map_or(false, |held| held.is_in_hand()),
            is_usable: item.satisfies::<Or<&Usable, &Medkit>>(),
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
        .query::<&Pos>()
        .iter()
        .filter(|&(_, pos)| pos.is_in(character_pos.get_area()))
        .map(|(entity, pos)| build_object_data(state, entity, pos))
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
        extra_background_layers: area.extra_background_layers.clone(),
        darkness: area.darkness,
        character_coord: character_pos.get_coord(),
        inventory,
        objects,
    }
}

fn build_object_data(state: &GameState, entity: Entity, pos: &Pos) -> ObjectRenderData {
    let entity_ref = state.world.entity(entity).unwrap();
    let properties = RenderProperties {
        direction: entity_ref
            .get::<&Direction>()
            .map(deref_clone)
            .unwrap_or_default(),
        aftik_color: entity_ref.get::<&AftikColorId>().map(deref_clone),
        is_cut: entity_ref.satisfies::<&IsCut>(),
        is_alive: entity_ref
            .get::<&Health>()
            .map_or(true, |health| health.is_alive()),
        is_badly_hurt: entity_ref
            .get::<&Health>()
            .map_or(false, |health| health.is_badly_hurt()),
    };
    ObjectRenderData {
        coord: pos.get_coord(),
        weight: entity_ref
            .get::<&OrderWeight>()
            .map(deref_clone)
            .unwrap_or_default(),
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
        name_data: ObjectNameData::build(entity_ref, &state.world),
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
