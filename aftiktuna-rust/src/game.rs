use specs::{prelude::*, storage::BTreeStorage, Component};
use std::cmp::max;

pub use position::{Coord, Position};

use crate::Messages;

mod position;

const AREA_SIZE: Coord = 5;

#[derive(Component, Debug)]
#[storage(BTreeStorage)]
pub struct GOType {
    symbol: char,
    name: String,
}

impl GOType {
    pub fn new(symbol: char, name: &str) -> GOType {
        GOType {
            symbol,
            name: String::from(name),
        }
    }
}

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct FuelCan;

pub struct AreaView;

impl<'a> System<'a> for AreaView {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, GOType>,
        WriteExpect<'a, Messages>,
    );

    fn run(&mut self, (pos, obj_type, mut messages): Self::SystemData) {
        let mut symbols_by_pos = init_symbol_vectors(AREA_SIZE);
        let mut labels = Vec::new();

        for (pos, obj_type) in (&pos, &obj_type).join() {
            symbols_by_pos[pos.get_coord()].push(obj_type.symbol);
            let label = format!("{}: {}", obj_type.symbol, obj_type.name);
            if !labels.contains(&label) {
                labels.push(label);
            }
        }

        println!("-----------");
        println!("Room:");
        let rows: usize = max(1, symbols_by_pos.iter().map(|vec| vec.len()).max().unwrap());
        for row in (0..rows).rev() {
            let base_symbol = if row == 0 { '_' } else { ' ' };
            let mut symbols = init_symbol_vector(AREA_SIZE, base_symbol);
            for pos in 0..AREA_SIZE {
                if let Some(symbol) = symbols_by_pos[pos].get(row) {
                    symbols[pos] = *symbol;
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
    }
}

fn init_symbol_vectors(size: usize) -> Vec<Vec<char>> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push(Vec::new());
    }
    symbols
}

fn init_symbol_vector(size: usize, symbol: char) -> Vec<char> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push(symbol);
    }
    symbols
}
