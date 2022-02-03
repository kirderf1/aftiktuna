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
            let (fuel_can, item_pos) =
                find_fuel_can(world.entities(), world.read_storage(), world.read_storage())
                    .expect("Expected a fuel can to exist");
            let mut pos = world.write_storage::<Position>();
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

fn find_fuel_can(
    entities: Entities,
    pos: ReadStorage<Position>,
    fuel_markers: ReadStorage<FuelCan>,
) -> Option<(Entity, Coord)> {
    // Return any entity with the "fuel can" marker
    (&entities, &pos, &fuel_markers)
        .join()
        .next()
        .map(|pair| (pair.0, pair.1.get_coord()))
}
