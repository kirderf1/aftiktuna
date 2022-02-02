use specs::{Component, storage::BTreeStorage};
use specs::prelude::*;

const AREA_SIZE: usize = 5;

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

#[derive(Component, Debug)]
#[storage(BTreeStorage)]
pub struct Position {
    coord: usize,
}

impl Position {
    pub fn new(coord: usize) -> Position {
        assert_valid_coord(coord);
        Position { coord }
    }

    pub fn get_pos(&self) -> usize {
        self.coord
    }

    pub fn move_to(&mut self, new_coord: usize) {
        assert_valid_coord(new_coord);
        self.coord = new_coord;
    }
}

fn assert_valid_coord(coord: usize) {
    if coord >= AREA_SIZE {
        panic!(
            "Position {} is out of bounds for room with size {}.",
            coord, AREA_SIZE
        );
    }
}

pub struct AreaView;

impl<'a> System<'a> for AreaView {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, GOType>);

    fn run(&mut self, (pos, obj_type): Self::SystemData) {
        let mut symbols = init_symbol_vector(AREA_SIZE);
        let mut labels = Vec::new();

        for (pos, obj_type) in (&pos, &obj_type).join() {
            symbols[pos.coord] = obj_type.symbol;
            labels.push(format!("{}: {}", obj_type.symbol, obj_type.name));
        }
        println!("{}", String::from_iter(symbols.iter()));
        for label in labels {
            println!("{}", label);
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
