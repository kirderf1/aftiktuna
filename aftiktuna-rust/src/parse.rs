use crate::action;
use crate::action::Action;
use crate::area::Position;
use crate::view::DisplayInfo;
use hecs::{Entity, Query, World};

pub fn try_parse_input(input: &str, world: &World, aftik: Entity) -> Result<Action, String> {
    let parse = Parse::new(input);
    parse
        .literal("take")
        .map(|parse| take(&parse, world, aftik))
        .or_else(|| {
            parse
                .literal("enter")
                .map(|parse| parse_enter(&parse, world, aftik))
        })
        .unwrap_or_else(|| Err(format!("Unexpected input: \"{}\"", input)))
}

fn take(parse: &Parse, world: &World, aftik: Entity) -> Result<Action, String> {
    parse.entity_from_remaining::<&action::Item, _, _>(world, aftik, action::take_item_action)
}

fn parse_enter(parse: &Parse, world: &World, aftik: Entity) -> Result<Action, String> {
    parse.entity_from_remaining::<&action::Door, _, _>(world, aftik, |door, _name| {
        action::enter_door_action(door)
    })
}

fn query_entity<T: Query>(aftik: Entity, door_type: &str, world: &World) -> Option<Entity> {
    let area = world.get::<Position>(aftik).unwrap().get_area();
    world
        .query::<(&Position, &DisplayInfo, T)>()
        .iter()
        .find(|(_, (pos, display_info, _))| {
            pos.get_area().eq(&area) && display_info.name().eq_ignore_ascii_case(door_type)
        })
        .map(|(entity, _)| entity)
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

    fn entity_from_remaining<Q, T, U>(&self, world: &World, aftik: Entity, closure: T) -> U
    where
        Q: Query,
        T: FnOnce(Option<Entity>, &str) -> U,
    {
        closure(query_entity::<Q>(aftik, self.input, world), self.input)
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
