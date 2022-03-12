use crate::action;
use crate::action::Action;
use crate::area::Position;
use crate::view::DisplayInfo;
use hecs::{Entity, Fetch, Query, World};

pub fn try_parse_input(input: &str, world: &World, aftik: Entity) -> Result<Action, String> {
    let parse = Parse::new(input);
    None.or_else(|| {
        parse
            .literal("take")
            .map(|parse| take(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("enter")
            .map(|parse| enter(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("force")
            .map(|parse| force(&parse, world, aftik))
    })
    .unwrap_or_else(|| Err(format!("Unexpected input: \"{}\"", input)))
}

fn take(parse: &Parse, world: &World, aftik: Entity) -> Result<Action, String> {
    None.or_else(|| {
        parse
            .literal("all")
            .and_then(|parse| parse.done(|| Ok(Action::TakeAll)))
    })
    .unwrap_or_else(|| {
        parse.entity_from_remaining::<&action::Item, _, _, _>(
            world,
            aftik,
            |item, _query, name| Ok(Action::TakeItem(item, name.to_string())),
            |name| Err(format!("There is no {} here to pick up.", name)),
        )
    })
}

fn enter(parse: &Parse, world: &World, aftik: Entity) -> Result<Action, String> {
    parse.entity_from_remaining::<&action::Door, _, _, _>(
        world,
        aftik,
        |door, _query, _name| Ok(Action::EnterDoor(door)),
        |_name| Err("There is no such door here to go through.".to_string()),
    )
}

fn force(parse: &Parse, world: &World, aftik: Entity) -> Result<Action, String> {
    parse.entity_from_remaining::<&action::Door, _, _, _>(
        world,
        aftik,
        |door, _query, _name| Ok(Action::ForceDoor(door)),
        |_name| Err("There is no such door here.".to_string()),
    )
}

fn query_entity<Q: Query, F, T>(
    aftik: Entity,
    door_type: &str,
    world: &World,
    on_match: F,
) -> Option<T>
where
    F: FnOnce(Entity, <<Q as Query>::Fetch as Fetch>::Item) -> T,
{
    let area = world.get::<Position>(aftik).unwrap().get_area();
    world
        .query::<(&Position, &DisplayInfo, Q)>()
        .iter()
        .find(|(_, (pos, display_info, _))| {
            pos.get_area().eq(&area) && display_info.name().eq_ignore_ascii_case(door_type)
        })
        .map(|(entity, (_, _, query))| on_match(entity, query))
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

    fn entity_from_remaining<Q, F, G, T>(
        &self,
        world: &World,
        aftik: Entity,
        on_match: F,
        on_none: G,
    ) -> T
    where
        Q: Query,
        F: FnOnce(Entity, <<Q as Query>::Fetch as Fetch>::Item, &str) -> T,
        G: FnOnce(&str) -> T,
    {
        query_entity::<Q, _, T>(aftik, self.input, world, |entity, query| {
            on_match(entity, query, self.input)
        })
        .unwrap_or_else(|| on_none(self.input))
    }
}
