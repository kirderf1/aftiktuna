use crate::action::{description, Door, DoorBlocking, InInventory};
use crate::area::{Area, Position};
use hecs::{Entity, World};
use std::cmp::max;

#[derive(Default)]
pub struct Messages(pub Vec<String>);

#[derive(Debug)]
pub struct DisplayInfo {
    symbol: char,
    name: String,
    weight: u32,
}

impl DisplayInfo {
    pub fn new(symbol: char, name: &str, weight: u32) -> DisplayInfo {
        DisplayInfo {
            symbol,
            name: String::from(name),
            weight,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub fn print_area_view(world: &World, aftik: Entity, messages: &mut Messages) {
    let area = get_viewed_area(aftik, world);
    let area_info = world.get::<Area>(area).unwrap();
    let area_size = area_info.size;

    let mut symbols_by_pos = init_symbol_vectors(area_size);
    let mut labels = Vec::new();

    for (entity, (pos, obj_type)) in world.query::<(&Position, &DisplayInfo)>().iter() {
        if pos.get_area() == area {
            symbols_by_pos[pos.get_coord()].push((obj_type.symbol, obj_type.weight));

            let label = format!(
                "{}: {}",
                obj_type.symbol,
                get_name(world, entity, &obj_type.name)
            );
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
    }

    for symbol_column in &mut symbols_by_pos {
        symbol_column.sort_by(|a, b| b.1.cmp(&a.1));
    }

    println!("-----------");
    println!("{}:", area_info.label);
    let rows: usize = max(1, symbols_by_pos.iter().map(|vec| vec.len()).max().unwrap());
    for row in (0..rows).rev() {
        let base_symbol = if row == 0 { '_' } else { ' ' };
        let mut symbols = vec![base_symbol; area_size];
        for pos in 0..area_size {
            if let Some(symbol) = symbols_by_pos[pos].get(row) {
                symbols[pos] = symbol.0;
            }
        }
        println!("{}", String::from_iter(symbols.iter()));
    }
    for label in labels {
        println!("{}", label);
    }
    println!();
    if !messages.0.is_empty() {
        println!("{}", messages.0.join(" "));
        messages.0.clear();
    }
    let inventory = world
        .query::<(&DisplayInfo, &InInventory)>()
        .iter()
        .map(|(_, (info, _))| info.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    if !inventory.is_empty() {
        println!("Inventory: {}", inventory);
    }
}

fn get_name(world: &World, entity: Entity, name: &str) -> String {
    if let Ok(door_pair) = world.get::<Door>(entity).map(|door| door.door_pair) {
        if let Ok(blocking) = world.get::<DoorBlocking>(door_pair) {
            return format!("{} ({})", name, description(blocking.0));
        }
    }
    name.to_string()
}

fn get_viewed_area(aftik: Entity, world: &World) -> Entity {
    world.get::<Position>(aftik).unwrap().get_area()
}

fn init_symbol_vectors<T>(size: usize) -> Vec<Vec<T>> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push(Vec::new());
    }
    symbols
}
