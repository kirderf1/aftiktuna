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

    let aftik = init_area(&mut world);

    AreaView.run_now(&world);

    loop {
        let input = read_input();

        if input.eq_ignore_ascii_case("take fuel can") {
            take_fuel_can(&mut world, aftik);

            println!("Congratulations, you won!");

            return;
        } else {
            println!("Unexpected input. \"{}\" is not \"take fuel can\"", input);
        }
    }
}

fn init_area(world: &mut World) -> Entity {
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
    aftik
}

fn read_input() -> String {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    String::from(input.trim())
}

fn take_fuel_can(world: &mut World, aftik: Entity) {
    let (fuel_can, item_pos) =
        find_fuel_can(world.entities(), world.read_storage(), world.read_storage())
            .expect("Expected a fuel can to exist");
    let mut pos = world.write_storage::<Position>();
    pos.get_mut(aftik).unwrap().move_to(item_pos);
    drop(pos);
    world.delete_entity(fuel_can).unwrap();

    AreaView.run_now(&world);
    println!("You picked up the fuel can.");
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
