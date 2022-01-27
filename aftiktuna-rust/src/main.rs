use game::{Area, GOType};

mod game;

fn main() {
    println!("Hello universe!");

    let aftik = GOType::new('A', "Aftik");
    let fuel_can = GOType::new('f', "Fuel can");
    let mut area = Area::new(5);
    area.add(1, aftik);
    area.add(4, fuel_can);

    area.print_area();
}
