use std::io;
use std::io::Write;

use specs::prelude::*;

use game::view::{AreaView, GOType, Messages};
use game::*;

mod game;

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    world.register::<GOType>();
    world.register::<Position>();
    world.register::<FuelCan>();
    world.insert(GameState::default());
    world.insert(Messages::default());

    let aftik = init_area(&mut world);
    world.fetch_mut::<GameState>().aftik = Some(aftik);

    loop {
        AreaView.run_now(&world);

        if world.fetch::<GameState>().has_won {
            println!("Congratulations, you won!");
            break;
        }

        loop {
            let input = read_input();

            if input.eq_ignore_ascii_case("take fuel can") {
                TakeFuelCan.run_now(&world);
                world.maintain();

                break;
            } else {
                println!("Unexpected input. \"{}\" is not \"take fuel can\"", input);
            }
        }
    }
}

#[derive(Default)]
pub struct GameState {
    has_won: bool,
    aftik: Option<Entity>,
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
