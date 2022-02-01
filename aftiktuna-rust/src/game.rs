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

pub struct Area {
    objects: Vec<GameObject>,
}

impl Area {
    pub fn new() -> Area {
        Area {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, pos: usize, obj_type: GOType) {
        if pos >= AREA_SIZE {
            panic!(
                "Position {} is out of bounds for room with size {}.",
                pos, AREA_SIZE
            );
        }
        self.objects.push(GameObject::new(obj_type, pos))
    }
}

struct GameObject {
    obj_type: GOType,
    pos: usize,
}

impl GameObject {
    pub fn new(obj_type: GOType, pos: usize) -> GameObject {
        GameObject { obj_type, pos }
    }
}

pub fn print_area(area: &Area) {
    let mut symbols = init_symbol_vector(AREA_SIZE);
    for obj in &area.objects {
        symbols[obj.pos] = obj.obj_type.symbol;
    }
    println!("{}", String::from_iter(symbols.iter()));
    for obj in &area.objects {
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
