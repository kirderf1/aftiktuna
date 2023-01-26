pub use status::print_full_status;

use crate::action::door::{Door, DoorBlocking};
use crate::action::trade;
use crate::area::Area;
use crate::position::{Coord, Pos};
use hecs::{Entity, World};
use std::cmp::max;
use std::collections::HashMap;

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
    name_data: NameData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameData {
    Name(String),
    Noun(String),
}

impl NameData {
    pub fn from_name(name: &str) -> Self {
        NameData::Name(name.to_string())
    }
    pub fn from_noun(noun: &str) -> Self {
        NameData::Noun(noun.to_string())
    }
    pub fn find(world: &World, entity: Entity) -> Self {
        world
            .get::<&DisplayInfo>(entity)
            .map(|info| info.name_data.clone())
            .unwrap_or_else(|_| NameData::Name("???".to_string()))
    }
}

pub fn group_data(data: Vec<NameData>) -> (Vec<String>, Vec<(String, i32)>) {
    let mut names = Vec::new();
    let mut nouns = HashMap::new();

    for name_data in data {
        match name_data {
            NameData::Name(name) => names.push(name),
            NameData::Noun(noun) => *nouns.entry(noun).or_insert(0) += 1,
        }
    }

    (names, nouns.into_iter().collect())
}

pub type StatusCache = status::Cache;

impl DisplayInfo {
    pub fn from_name(symbol: char, name: &str, weight: u32) -> DisplayInfo {
        DisplayInfo {
            symbol,
            name_data: NameData::from_name(name),
            weight,
        }
    }

    pub fn from_noun(symbol: char, noun: &str, weight: u32) -> DisplayInfo {
        DisplayInfo {
            symbol,
            name_data: NameData::from_noun(noun),
            weight,
        }
    }

    pub fn name(&self) -> &str {
        match &self.name_data {
            NameData::Name(name) | NameData::Noun(name) => name,
        }
    }

    pub fn definite_name(&self) -> String {
        match &self.name_data {
            NameData::Name(name) => name.to_string(),
            NameData::Noun(name) => format!("the {}", name),
        }
    }

    pub fn matches(&self, string: &str) -> bool {
        self.name().eq_ignore_ascii_case(string)
    }

    pub fn find_definite_name(world: &World, entity: Entity) -> String {
        world
            .get::<&DisplayInfo>(entity)
            .map_or_else(|_| "???".to_string(), |info| info.definite_name())
    }

    pub fn find_name(world: &World, entity: Entity) -> String {
        world
            .get::<&DisplayInfo>(entity)
            .map_or_else(|_| "???".to_string(), |info| info.name().to_string())
    }
}

pub fn print(world: &World, character: Entity, messages: &mut Messages, cache: &mut StatusCache) {
    println!("-----------");
    if let Some(shopkeeper) = trade::get_shop_info(world, character) {
        let items = shopkeeper
            .0
            .iter()
            .map(|priced| (capitalize(priced.item.display_info().name()), priced.price))
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
                get_name(world, entity, &capitalize(obj_type.name()))
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
    for label in labels {
        println!("{}", label);
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

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
