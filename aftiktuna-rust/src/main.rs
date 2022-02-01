use specs::prelude::*;

use game::{AreaView, GOType, Position};

mod game;

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    world.register::<GOType>();
    world.register::<Position>();

    let aftik = GOType::new('A', "Aftik");
    let fuel_can = GOType::new('f', "Fuel can");

    world
        .create_entity()
        .with(aftik)
        .with(Position::new(1))
        .build();
    world
        .create_entity()
        .with(fuel_can)
        .with(Position::new(4))
        .build();

    AreaView.run_now(&world);
}
