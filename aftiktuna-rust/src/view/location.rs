use crate::action::door::{BlockType, Door};
use crate::action::item::get_wielded;
use crate::area::{Area, BackgroundType};
use crate::item;
use crate::position::{Coord, Direction, Pos};
use crate::view::{capitalize, DisplayInfo, Messages, NameData};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::cmp::max;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TextureType {
    Unknown,
    SmallUnknown,
    FortunaChest,
    Ship,
    Door,
    CutDoor,
    ShipExit,
    Shack,
    CutShack,
    Path,
    Aftik,
    Goblin,
    Eyesaur,
    Azureclops,
    Item(item::Type),
}

impl From<item::Type> for TextureType {
    fn from(value: item::Type) -> Self {
        TextureType::Item(value)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AftikColor {
    Mint,
    Cerulean,
    Plum,
    Green,
}

pub fn area_view_messages(render_data: &RenderData) -> Messages {
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
        symbols_by_pos[object.coord].push((object.symbol, object.weight));

        let label = format!("{}: {}", object.symbol, object.name,);
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
            if let Some(symbol) = symbols_by_pos[pos].get(row) {
                symbols[pos] = symbol.0;
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
    pub background: Option<BackgroundType>,
    pub background_offset: Option<Coord>,
    pub character_coord: Coord,
    pub objects: Vec<ObjectRenderData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRenderData {
    pub coord: Coord,
    pub weight: u32,
    pub texture_type: TextureType,
    pub name: String,
    pub symbol: char,
    pub direction: Direction,
    pub aftik_color: Option<AftikColor>,
    pub wielded_item: Option<TextureType>,
}

pub(crate) fn prepare_render_data(world: &World, character: Entity) -> RenderData {
    let character_pos = world.get::<&Pos>(character).unwrap();
    let area = world.get::<&Area>(character_pos.get_area()).unwrap();

    let mut objects: Vec<ObjectRenderData> = world
        .query::<(&Pos, &DisplayInfo, Option<&Direction>, Option<&AftikColor>)>()
        .iter()
        .filter(|(_, (pos, _, _, _))| pos.is_in(character_pos.get_area()))
        .map(
            |(entity, (pos, display_info, direction, color))| ObjectRenderData {
                coord: pos.get_coord(),
                weight: display_info.weight,
                texture_type: display_info.texture_type,
                name: get_name(
                    world,
                    entity,
                    capitalize(NameData::find(world, entity).base()),
                ),
                symbol: display_info.symbol,
                direction: direction.copied().unwrap_or(Direction::Right),
                aftik_color: color.copied(),
                wielded_item: find_wielded_item_texture(world, entity),
            },
        )
        .collect();
    objects.sort_by(|data1, data2| data2.weight.cmp(&data1.weight));

    RenderData {
        area_label: area.label.clone(),
        area_size: area.size,
        background: area.background,
        background_offset: area.background_offset,
        character_coord: character_pos.get_coord(),
        objects,
    }
}

fn find_wielded_item_texture(world: &World, holder: Entity) -> Option<TextureType> {
    let item = get_wielded(world, holder)?;
    world
        .get::<&DisplayInfo>(item)
        .ok()
        .map(|info| info.texture_type)
}
