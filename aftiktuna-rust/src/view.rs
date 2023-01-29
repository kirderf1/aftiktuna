pub use status::print_full_status;

use crate::action::door::{Door, DoorBlocking};
use crate::action::trade;
use crate::area::Area;
use crate::position::{Coord, Pos};
use hecs::{Entity, World};
pub use name::{as_grouped_text_list, NounData};
use std::cmp::max;

mod name;
mod status;

#[derive(Default)]
pub struct Messages(Vec<String>);

impl Messages {
    pub fn simple(message: String) -> Messages {
        let mut messages = Messages::default();
        messages.add(message);
        messages
    }

    pub fn add(&mut self, message: String) {
        self.0.push(message);
    }

    pub fn print_and_clear(&mut self) {
        if !self.0.is_empty() {
            println!(
                "{}",
                self.0
                    .iter()
                    .map(|line| capitalize(line))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            self.0.clear();
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayInfo {
    symbol: char,
    weight: u32,
}

pub type StatusCache = status::Cache;

pub type NameData = name::Data;

impl DisplayInfo {
    pub fn new(symbol: char, weight: u32) -> Self {
        DisplayInfo { symbol, weight }
    }
}

pub fn print(world: &World, character: Entity, messages: &mut Messages, cache: &mut StatusCache) {
    println!("-----------");
    if let Some(shopkeeper) = trade::get_shop_info(world, character) {
        let items = shopkeeper
            .0
            .iter()
            .map(|priced| (capitalize(priced.item.noun_data().singular()), priced.price))
            .collect::<Vec<_>>();
        let max_length = items.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        for (name, price) in items {
            println!(
                "{} {}| {}p",
                name,
                " ".repeat(max_length - name.len()),
                price
            );
        }
        status::print_points(world, character, cache);
    } else {
        let area = get_viewed_area(character, world);
        let area_info = world.get::<&Area>(area).unwrap();
        let area_size = area_info.size;
        println!("{}:", area_info.label);
        print_area(world, area, area_size);
    }

    println!();
    messages.print_and_clear();
    status::print_changes(world, character, cache);
}

fn print_area(world: &World, area: Entity, area_size: Coord) {
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
                    &capitalize(NameData::find(world, entity).base())
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
        println!("{}", symbols.iter().collect::<String>());
    }
    for label in labels.chunks(3) {
        println!("{}", label.join("   "));
    }
}

fn get_name(world: &World, entity: Entity, name: &str) -> String {
    if let Ok(door_pair) = world.get::<&Door>(entity).map(|door| door.door_pair) {
        if let Ok(blocking) = world.get::<&DoorBlocking>(door_pair) {
            return format!("{} ({})", name, blocking.0.description());
        }
    }
    name.to_string()
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

pub fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn name_display_info(name: &str) -> DisplayInfo {
    DisplayInfo::new(name.chars().next().unwrap(), 10)
}
