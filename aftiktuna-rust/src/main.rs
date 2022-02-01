use game::{GameObject, GOType};

mod game;

fn main() {
    println!("Hello universe!");

    let aftik = GOType::new('A', "Aftik");
    let fuel_can = GOType::new('f', "Fuel can");
    let mut area = Vec::new();
    area.push(GameObject::new(aftik, 1));
    area.push(GameObject::new(fuel_can, 4));

    game::print_area(&area);
}
