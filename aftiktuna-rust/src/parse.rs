use crate::{parse_enter_door, parse_take_fuel_can, Action};
use hecs::{Entity, World};

pub fn try_parse_input(input: &str, world: &World, aftik: Entity) -> Result<Action, String> {
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
