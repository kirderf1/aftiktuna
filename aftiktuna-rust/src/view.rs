pub use status::print_full_status;

use crate::action::door::{BlockType, Door};
use crate::action::trade;
use crate::area::Area;
use crate::game_loop::StopType;
use crate::item;
use crate::position::{Coord, Direction, Pos};
use hecs::{Entity, World};
pub use name::{as_grouped_text_list, NounData};
use std::cmp::max;
use std::mem::take;

mod name;
mod status;

#[derive(Default)]
pub struct Messages(Vec<String>);

impl Messages {
    pub fn add(&mut self, message: impl AsRef<str>) {
        self.0.push(capitalize(message));
    }

    pub fn print_lines(self) {
        for line in self.0 {
            println!("{line}");
        }
    }

    pub fn into_text(self) -> Vec<String> {
        self.0
    }
}

impl<T: AsRef<str>> From<T> for Messages {
    fn from(value: T) -> Self {
        let mut messages = Self::default();
        messages.add(value);
        messages
    }
}

#[derive(Clone, Debug)]
pub struct DisplayInfo {
    symbol: char,
    texture_type: TextureType,
    weight: u32,
}

pub type StatusCache = status::Cache;

pub type NameData = name::Data;

#[derive(Default)]
pub struct Buffer {
    pub messages: Messages,
    captured_frames: Vec<Frame>,
}

impl Buffer {
    pub fn capture_view(&mut self, world: &World, character: Entity, cache: &mut StatusCache) {
        let mut messages = Messages::default();
        self.pop_message_cache(&mut messages);
        status::changes_messages(world, character, &mut messages, cache);

        self.captured_frames.push(Frame::Full {
            view: view_messages(world, character, cache),
            messages,
            render_data: prepare_render_data(world, character),
        });
    }

    fn pop_message_cache(&mut self, messages: &mut Messages) {
        let messages_text = take(&mut self.messages).0.join(" ");
        if !messages_text.is_empty() {
            messages.add(messages_text);
        }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.captured_frames.push(frame);
    }

    pub fn into_frames(self) -> Vec<Frame> {
        self.captured_frames
    }
}

pub enum Frame {
    Full {
        view: Messages,
        messages: Messages,
        render_data: RenderData,
    },
    LocationChoice(Messages),
    Ending(StopType),
}

impl Frame {
    pub fn as_text(&self) -> Vec<String> {
        let mut text = Vec::new();
        match self {
            Frame::Full { view, messages, .. } => {
                text.push("--------------------".to_string());
                text.extend(view.0.clone());

                if !messages.0.is_empty() {
                    text.push(String::default());

                    text.extend(messages.0.clone());
                }
            }
            Frame::LocationChoice(messages) => {
                text.push("--------------------".to_string());
                text.extend(messages.0.clone());
            }
            Frame::Ending(stop_type) => {
                text.push(String::default());
                text.extend(stop_type.messages().into_text());
            }
        }
        text
    }
}

impl DisplayInfo {
    pub fn new(symbol: char, texture_type: TextureType, weight: u32) -> Self {
        DisplayInfo {
            symbol,
            texture_type,
            weight,
        }
    }
}

fn view_messages(world: &World, character: Entity, cache: &mut StatusCache) -> Messages {
    let mut messages = Messages::default();
    if let Some(shopkeeper) = trade::get_shop_info(world, character) {
        let items = shopkeeper
            .0
            .iter()
            .map(|priced| (capitalize(priced.item.noun_data().singular()), priced.price))
            .collect::<Vec<_>>();
        let max_length = items.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        for (name, price) in items {
            messages.add(format!(
                "{} {}| {}p",
                name,
                " ".repeat(max_length - name.len()),
                price
            ));
        }
        status::print_points(world, character, &mut messages, cache);
    } else {
        let area = get_viewed_area(character, world);
        let area_info = world.get::<&Area>(area).unwrap();
        let area_size = area_info.size;
        messages.add(format!("{}:", area_info.label));
        print_area(world, &mut messages.0, area, area_size);
    }
    messages
}

fn print_area(world: &World, lines: &mut Vec<String>, area: Entity, area_size: Coord) {
    let mut symbols_by_pos = init_symbol_vectors(area_size);
    let mut labels = Vec::new();

    for (entity, (pos, obj_type)) in world.query::<(&Pos, &DisplayInfo)>().iter() {
        if pos.get_area() == area {
            symbols_by_pos[pos.get_coord()].push((obj_type.symbol, obj_type.weight));

            let label = format!(
                "{}: {}",
                obj_type.symbol,
                get_name(
                    world,
                    entity,
                    capitalize(NameData::find(world, entity).base())
                )
            );
            if !labels.contains(&label) {
                labels.push(label);
            }
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

fn get_name(world: &World, entity: Entity, name: String) -> String {
    if let Ok(door_pair) = world.get::<&Door>(entity).map(|door| door.door_pair) {
        if let Ok(blocking) = world.get::<&BlockType>(door_pair) {
            return format!("{} ({})", name, blocking.description());
        }
    }
    name
}

fn get_viewed_area(aftik: Entity, world: &World) -> Entity {
    world.get::<&Pos>(aftik).unwrap().get_area()
}

fn init_symbol_vectors<T>(size: usize) -> Vec<Vec<T>> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push(Vec::new());
    }
    symbols
}

pub fn capitalize(text: impl AsRef<str>) -> String {
    let mut chars = text.as_ref().chars();
    match chars.next() {
        None => String::new(),
        Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn name_display_info(texture_type: TextureType, name: &str) -> DisplayInfo {
    DisplayInfo::new(name.chars().next().unwrap(), texture_type, 10)
}

pub struct RenderData {
    pub size: Coord,
    pub objects: Vec<ObjectRenderData>,
}

pub struct ObjectRenderData {
    pub coord: Coord,
    pub texture_type: TextureType,
    pub direction: Direction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TextureType {
    Unknown,
    SmallUnknown,
    Door,
    Ship,
    Path,
    Aftik,
    Goblin,
    Eyesaur,
    Azureclops,
    Item(item::Type),
}

fn prepare_render_data(world: &World, character: Entity) -> RenderData {
    let area = get_viewed_area(character, world);
    let size = world.get::<&Area>(area).unwrap().size;

    let objects = world
        .query::<(&Pos, &DisplayInfo, Option<&Direction>)>()
        .iter()
        .filter(|(_, (pos, _, _))| pos.is_in(area))
        .map(|(_, (pos, display_info, direction))| ObjectRenderData {
            coord: pos.get_coord(),
            texture_type: display_info.texture_type,
            direction: direction.copied().unwrap_or(Direction::Right),
        })
        .collect();

    RenderData { size, objects }
}
