use hecs::{Entity, World};
use std::io;
use std::io::Write;

use crate::action::FuelCan;
use action::Action;
use view::Messages;

mod action;
mod area;
mod parse;
mod view;

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    let mut messages = Messages::default();

    let aftik = area::init_area(&mut world);

    loop {
        view::print_area_view(&world, aftik, &mut messages);

        if action::has_item::<FuelCan>(&world) {
            println!("Congratulations, you won!");
            break;
        }

        let action = parse_user_action(&world, aftik);
        action::run_action(action, &mut world, aftik, &mut messages);
    }
}

fn parse_user_action(world: &World, aftik: Entity) -> Action {
    loop {
        let input = read_input().to_lowercase();

        match parse::try_parse_input(&input, world, aftik) {
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
