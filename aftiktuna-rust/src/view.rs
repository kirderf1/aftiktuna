pub use status::print_full_status;

use crate::action::door::{BlockType, Door};
use crate::action::trade;
use crate::area::Area;
use crate::position::{Coord, Pos};
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

    pub fn print(self) {
        if !self.0.is_empty() {
            println!("{}", self.0.join(" "));
        }
    }

    pub fn print_lines(self) {
        for line in self.0 {
            println!("{line}");
        }
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
    weight: u32,
}

pub type StatusCache = status::Cache;

pub type NameData = name::Data;

pub struct Data {
    view: Messages,
    messages: Messages,
    changes: Messages,
}

impl Data {
    pub fn print(self) {
        println!("-----------");

        self.view.print_lines();

        println!();

        self.messages.print();

        self.changes.print_lines();
    }
}

impl DisplayInfo {
    pub fn new(symbol: char, weight: u32) -> Self {
        DisplayInfo { symbol, weight }
    }
}

pub fn capture(
    world: &World,
    character: Entity,
    messages: &mut Messages,
    cache: &mut StatusCache,
) -> Data {
    Data {
        view: view_messages(world, character, cache),
        messages: take(messages),
        changes: status::changes_messages(world, character, cache),
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
        print_area(world, &mut messages, area, area_size);
    }
    messages
}

fn print_area(world: &World, messages: &mut Messages, area: Entity, area_size: Coord) {
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
        messages.add(symbols.iter().collect::<String>());
    }
    for labels in labels.chunks(3) {
        messages.add(labels.join("   "));
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

pub fn name_display_info(name: &str) -> DisplayInfo {
    DisplayInfo::new(name.chars().next().unwrap(), 10)
}
