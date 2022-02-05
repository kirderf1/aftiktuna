use specs::{Component, prelude::*, storage::BTreeStorage};

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
        let mut symbols = init_symbol_vector(AREA_SIZE);
        let mut labels = Vec::new();

        for (pos, obj_type) in (&pos, &obj_type).join() {
            symbols[pos.get_coord()] = obj_type.symbol;
            labels.push(format!("{}: {}", obj_type.symbol, obj_type.name));
        }

        println!("-----------");
        println!("Room:");
        println!("{}", String::from_iter(symbols.iter()));
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

fn init_symbol_vector(size: usize) -> Vec<char> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push('_');
    }
    symbols
}
