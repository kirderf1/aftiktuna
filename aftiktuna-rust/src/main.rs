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
        let input = read_input().to_lowercase();

        match try_parse_input(&input, world, game_state.aftik) {
            Ok(action) => return action,
            Err(message) => println!("{}", message),
        }
    }
}

fn try_parse_input(input: &str, world: &World, aftik: Entity) -> Result<Action, String> {
    if input.eq("take fuel can") {
        parse_take_fuel_can(world, aftik)
    } else if let Some(result) = parse_enter(Parse::new(input), world, aftik) {
        result
    } else {
        Err(format!("Unexpected input: \"{}\"", input))
    }
}

fn parse_enter(parse: Parse, world: &World, aftik: Entity) -> Option<Result<Action, String>> {
    parse
        .literal("enter")?
        .match_remaining(&["door", "left door", "right door"], |door_type| {
            parse_enter_door(world, door_type, aftik)
        })
}

struct Parse<'a> {
    input: &'a str,
}

impl<'a> Parse<'a> {
    fn new(input: &str) -> Parse {
        Parse { input }
    }

    fn literal(&self, word: &str) -> Option<Parse<'a>> {
        if self.input.starts_with(word) {
            Some(Parse {
                input: self.input.split_at(word.len()).1.trim_start(),
            })
        } else {
            None
        }
    }

    fn match_remaining<T, U>(&self, words: &[&str], closure: T) -> Option<U>
    where
        T: FnOnce(&str) -> U,
    {
        for word in words {
            if self.input.eq(*word) {
                return Some(closure(word));
            }
        }
        None
    }

    fn done<T, U>(&self, closure: T) -> Option<U>
    where
        T: FnOnce() -> U,
    {
        if self.input.is_empty() {
            Some(closure())
        } else {
            None
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
