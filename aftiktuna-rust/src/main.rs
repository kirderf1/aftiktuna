use hecs::{Entity, World};
use std::io;
use std::io::Write;

use action::Action;
use view::Messages;

mod action;
mod area;
mod parse;
mod view;

pub struct GameState {
    aftik: Entity,
}

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    let mut messages = Messages::default();

    let aftik = area::init_area(&mut world);
    let game_state = GameState { aftik };

    loop {
        view::print_area_view(&world, &game_state, &mut messages);

        if action::has_fuel_can(&world) {
            println!("Congratulations, you won!");
            break;
        }

        let action = parse_user_action(&world, &game_state);
        action::run_action(action, &mut world, &game_state, &mut messages);
    }
}

fn parse_user_action(world: &World, game_state: &GameState) -> Action {
    loop {
        let input = read_input().to_lowercase();

        match parse::try_parse_input(&input, world, game_state.aftik) {
            Ok(action) => return action,
            Err(message) => println!("{}", message),
        }
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
