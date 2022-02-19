use hecs::{Entity, World};
use std::io;
use std::io::Write;

use action::*;
use area::*;
use view::*;

mod action;
mod area;
mod view;

pub struct GameState {
    has_won: bool,
    aftik: Entity,
}

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    let mut messages = Messages::default();

    let aftik = init_area(&mut world);
    let mut game_state = GameState {
        has_won: false,
        aftik,
    };

    loop {
        print_area_view(&world, &game_state, &mut messages);

        if game_state.has_won {
            println!("Congratulations, you won!");
            break;
        }

        let action = parse_user_action(&world, &game_state);
        run_action(action, &mut world, &mut game_state, &mut messages);
    }
}

fn parse_user_action(world: &World, game_state: &GameState) -> Action {
    loop {
        let input = read_input();

        match try_parse_input(input, world, game_state.aftik) {
            Ok(action) => return action,
            Err(message) => println!("{}", message),
        }
    }
}

fn try_parse_input(input: String, world: &World, aftik: Entity) -> Result<Action, String> {
    if input.eq_ignore_ascii_case("take fuel can") {
        parse_take_fuel_can(world, aftik)
    } else if input.eq_ignore_ascii_case("enter door") {
        parse_enter_door(world, aftik)
    } else {
        Err(format!(
            "Unexpected input. \"{}\" is not \"take fuel can\"",
            input
        ))
    }
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
