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
    world.insert(GameState {
        has_won: false,
        aftik: None,
    });
    world.insert(Messages(Vec::new()));

    let aftik = init_area(&mut world);
    world.fetch_mut::<GameState>().aftik = Some(aftik);

    AreaView.run_now(&world);

    loop {
        let input = read_input();

        if input.eq_ignore_ascii_case("take fuel can") {
            take_fuel_can(&mut world);

            AreaView.run_now(&world);

            if world.fetch::<GameState>().has_won {
                println!("Congratulations, you won!");
                return;
            }
        } else {
            println!("Unexpected input. \"{}\" is not \"take fuel can\"", input);
        }
    }
}

struct GameState {
    has_won: bool,
    aftik: Option<Entity>,
}

pub struct Messages(Vec<String>);

fn init_area(world: &mut World) -> Entity {
    let aftik = world
        .create_entity()
        .with(GOType::new('A', "Aftik"))
        .with(Position::new(1))
        .build();
    //place_fuel(world, 3);
    //place_fuel(world, 4);
    aftik
}

fn place_fuel(world: &mut World, pos: Coord) {
    world
        .create_entity()
        .with(GOType::new('f', "Fuel can"))
        .with(Position::new(pos))
        .with(FuelCan)
        .build();
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

fn take_fuel_can(world: &mut World) {
    let optional = find_fuel_can(world.entities(), world.read_storage(), world.read_storage());

    match optional {
        Some((fuel_can, item_pos)) => {
            let mut pos = world.write_storage::<Position>();
            let aftik = world
                .fetch::<GameState>()
                .aftik
                .expect("Expected aftik to be initialized");
            pos.get_mut(aftik).unwrap().move_to(item_pos);
            drop(pos);
            world.delete_entity(fuel_can).unwrap();
            world.fetch_mut::<GameState>().has_won = true;

            world
                .fetch_mut::<Messages>()
                .0
                .push("You picked up the fuel can.".to_string());
        }
        None => {
            world
                .fetch_mut::<Messages>()
                .0
                .push("There is no fuel can here to pick up.".to_string());
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
