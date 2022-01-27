fn main() {
    println!("Hello universe!");

    let aftik = GOType::new('A', "Aftik");
    let fuel_can = GOType::new('f', "Fuel can");
    let mut area = Area::new(5);
    area.add(1, aftik);
    area.add(4, fuel_can);

    print_area(&area);
}

fn print_area(area: &Area) {
    let mut symbols = init_symbol_vector(area.size);
    for obj in &area.objects {
        symbols[obj.pos] = obj.obj_type.symbol;
    }
    println!("{}", String::from_iter(symbols.iter()));
    for obj in &area.objects {
        let t = &obj.obj_type;
        println!("{}: {}", t.symbol, t.name);
    }
}

fn init_symbol_vector(size : usize) -> Vec<char> {
    let mut symbols = Vec::with_capacity(size);
    for _ in 0..size {
        symbols.push('_');
    }
    symbols
}

struct Area {
    size : usize,
    objects : Vec<GameObject>
}

impl Area {
    fn new(size : usize) -> Area {
       Area {
           size,
           objects : Vec::new()
       }
    }

    fn add(&mut self, pos : usize, obj_type : GOType) {
        if pos >= self.size {
            panic!("Position {} is out of bounds for room with size {}.", pos, self.size);
        }
        self.objects.push(GameObject::new(obj_type, pos))
    }
}

struct GameObject {
    obj_type : GOType,
    pos : usize
}

impl GameObject {
    fn new(obj_type : GOType, pos : usize) -> GameObject {
        GameObject {
            obj_type,
            pos
        }
    }
}

struct GOType {
    symbol : char,
    name : String
}

impl GOType {
    fn new(symbol : char, name : &str) -> GOType {
        GOType {
            symbol,
            name: String::from(name)
        }
    }
}