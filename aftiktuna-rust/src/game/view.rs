use crate::{Area, GameState, Position, ReadExpect, System, WriteExpect};
use specs::{storage::BTreeStorage, Component, Entity, Join, ReadStorage};
use std::cmp::max;
use std::ops::Deref;

#[derive(Default)]
pub struct Messages(pub Vec<String>);

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

pub struct AreaView;

impl<'a> System<'a> for AreaView {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, GOType>,
        ReadStorage<'a, Area>,
        ReadExpect<'a, GameState>,
        WriteExpect<'a, Messages>,
    );

    fn run(&mut self, (pos, obj_type, areas, game_state, mut messages): Self::SystemData) {
        let area = get_viewed_area(game_state.deref(), &pos);
        let area_info = areas.get(area).unwrap();
        let area_size = area_info.size;

        let mut symbols_by_pos = init_symbol_vectors(area_size);
        let mut labels = Vec::new();

        for (pos, obj_type) in (&pos, &obj_type).join() {
            symbols_by_pos[pos.get_coord()].push(obj_type.symbol);
            let label = format!("{}: {}", obj_type.symbol, obj_type.name);
            if !labels.contains(&label) {
                labels.push(label);
            }
        }

        println!("-----------");
        println!("{}:", area_info.label);
        let rows: usize = max(1, symbols_by_pos.iter().map(|vec| vec.len()).max().unwrap());
        for row in (0..rows).rev() {
            let base_symbol = if row == 0 { '_' } else { ' ' };
            let mut symbols = init_symbol_vector(area_size, base_symbol);
            for pos in 0..area_size {
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

fn get_viewed_area(game_state: &GameState, pos: &ReadStorage<Position>) -> Entity {
    let aftik = game_state.aftik.unwrap();
    pos.get(aftik).unwrap().get_area()
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
