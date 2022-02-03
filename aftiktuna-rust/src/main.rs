use std::io;
use std::io::Write;

use specs::prelude::*;

use game::*;

mod game;

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    world.register::<GOType>();
    world.register::<Position>();
    world.register::<FuelCan>();

    let aftik = GOType::new('A', "Aftik");
    let fuel_can = GOType::new('f', "Fuel can");

    let aftik = world
        .create_entity()
        .with(aftik)
        .with(Position::new(1))
        .build();
    world
        .create_entity()
        .with(fuel_can)
        .with(Position::new(4))
        .with(FuelCan)
        .build();

    AreaView.run_now(&world);

    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let input = input.trim();

        if input.eq_ignore_ascii_case("take fuel can") {
            let fuel_can = find_fuel_can(&world).expect("Expected a fuel can to exist");
            let mut pos = world.write_storage::<Position>();
            let item_pos = pos.get(fuel_can).unwrap().get_pos();
            pos.get_mut(aftik).unwrap().move_to(item_pos);
            drop(pos);
            world.delete_entity(fuel_can).unwrap();

            AreaView.run_now(&world);
            println!("You picked up the fuel can.");
            println!("Congratulations, you won!");

            return;
        } else {
            println!("Unexpected input. \"{}\" is not \"take fuel can\"", input);
        }
    }
}

fn find_fuel_can(world :&World) -> Option<Entity> {
    let fuel_cans = world.read_storage::<FuelCan>();
    let entities = world.entities();
    // Return any entity with the "fuel can" marker
    (&entities, &fuel_cans).join().next().map(|pair| pair.0)
}