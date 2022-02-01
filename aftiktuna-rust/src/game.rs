const AREA_SIZE: usize = 5;

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

pub struct Position {
    coord: usize,
}

impl Position {
    pub fn new(coord: usize) -> Position {
        if coord >= AREA_SIZE {
            panic!(
                "Position {} is out of bounds for room with size {}.",
                coord, AREA_SIZE
            );
        }
        Position { coord }
    }
}

pub struct GameObject {
    obj_type: GOType,
    pos: Position,
}

impl GameObject {
    pub fn new(obj_type: GOType, coord: usize) -> GameObject {
        GameObject {
            obj_type,
            pos: Position::new(coord),
        }
    }
}

pub fn print_area(area: &[GameObject]) {
    let mut symbols = init_symbol_vector(AREA_SIZE);
    for obj in area {
        symbols[obj.pos.coord] = obj.obj_type.symbol;
    }
    println!("{}", String::from_iter(symbols.iter()));
    for obj in area {
        let t = &obj.obj_type;
        println!("{}: {}", t.symbol, t.name);
    }
}

fn init_symbol_vector(size: usize) -> Vec<char> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push('_');
    }
    symbols
}
