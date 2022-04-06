use hecs::{Entity, World};
use std::io::Write;
use std::{io, thread, time};

use action::{item, Action};
use view::Messages;

mod action;
mod area;
mod parse;
mod position;
mod view;

fn main() {
    println!("Hello universe!");

    let mut world = World::new();
    let mut messages = Messages::default();
    let mut cache = None;

    let aftik = area::init_area(&mut world);

    loop {
        view::print(&world, aftik, &mut messages, &mut cache);

        if item::has_item::<item::FuelCan>(&world) {
            println!("Congratulations, you won!");
            break;
        }

        if world.get::<Action>(aftik).is_err() {
            let action = parse_user_action(&world, aftik);
            world.insert_one(aftik, action).unwrap();
        } else {
            thread::sleep(time::Duration::from_secs(2));
        }

        action::run_action(&mut world, aftik, &mut messages);
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
