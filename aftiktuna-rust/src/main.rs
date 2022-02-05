use std::io;
use std::io::Write;
use std::ops::Deref;

use specs::prelude::*;
use specs::shred::FetchMut;
use specs::storage::MaskedStorage;

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
            TakeFuelCan.run_now(&world);
            world.maintain();

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

struct TakeFuelCan;

impl<'a> System<'a> for TakeFuelCan {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, FuelCan>,
        WriteExpect<'a, GameState>,
        WriteExpect<'a, Messages>,
    );

    fn run(
        &mut self,
        (entities, mut pos, fuel_markers, mut game_state, mut messages): Self::SystemData,
    ) {
        let optional = find_fuel_can(&entities, &pos, &fuel_markers);

        match optional {
            Some((fuel_can, item_pos)) => {
                let aftik = game_state.aftik.expect("Expected aftik to be initialized");
                pos.get_mut(aftik).unwrap().move_to(item_pos);
                entities.delete(fuel_can).unwrap();
                game_state.has_won = true;

                messages.0.push("You picked up the fuel can.".to_string());
            }
            None => {
                messages
                    .0
                    .push("There is no fuel can here to pick up.".to_string());
            }
        }
    }
}

fn find_fuel_can<'a, P>(
    entities: &Entities,
    pos: &Storage<'a, Position, P>, //Any kind of position storage, could be either a WriteStorage<> or a ReadStorage<>
    fuel_markers: &ReadStorage<FuelCan>,
) -> Option<(Entity, Coord)>
where
    P: Deref<Target = MaskedStorage<Position>>,
{
    // Return any entity with the "fuel can" marker
    (entities, pos, fuel_markers)
        .join()
        .next()
        .map(|pair| (pair.0, pair.1.get_coord()))
}
