use crate::action::item::InInventory;
use crate::action::{combat, door, item, Action};
use crate::position::Pos;
use crate::view;
use crate::view::DisplayInfo;
use hecs::{Entity, With, World};

pub fn try_parse_input(
    input: &str,
    world: &World,
    aftik: Entity,
) -> Result<Option<Action>, String> {
    let parse = Parse::new(input);
    None.or_else(|| {
        parse
            .literal("take")
            .map(|parse| take(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("wield")
            .map(|parse| wield(&parse, world, aftik))
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
    .or_else(|| {
        parse
            .literal("attack")
            .map(|parse| attack(&parse, world, aftik))
    })
    .or_else(|| {
        parse
            .literal("status")
            .map(|parse| status(&parse, world, aftik))
    })
    .unwrap_or_else(|| Err(format!("Unexpected input: \"{}\"", input)))
}

fn take(parse: &Parse, world: &World, aftik: Entity) -> Result<Option<Action>, String> {
    None.or_else(|| {
        parse
            .literal("all")
            .and_then(|parse| parse.done(|| Ok(Some(Action::TakeAll))))
    })
    .unwrap_or_else(|| {
        parse.take_remaining(|name| {
            let aftik_pos = *world.get::<Pos>(aftik).unwrap();
            world
                .query::<With<item::Item, (&Pos, &DisplayInfo)>>()
                .iter()
                .filter(|(_, (pos, display_info))| {
                    pos.is_in(aftik_pos.get_area()) && display_info.matches(name)
                })
                .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
                .map(|(item, (_, display_info))| {
                    Ok(Some(Action::TakeItem(
                        item,
                        display_info.definite_name().to_string(),
                    )))
                })
                .unwrap_or_else(|| Err(format!("There is no {} here to pick up.", name)))
        })
    })
}

fn wield(parse: &Parse, world: &World, aftik: Entity) -> Result<Option<Action>, String> {
    parse.take_remaining(|name| {
        None.or_else(|| {
            world
                .query::<With<item::Item, With<InInventory, &DisplayInfo>>>()
                .iter()
                .find(|(_, display_info)| display_info.matches(name))
                .map(|(item, display_info)| {
                    Ok(Some(Action::Wield(
                        item,
                        display_info.definite_name().to_string(),
                    )))
                })
        })
        .or_else(|| {
            let aftik_pos = *world.get::<Pos>(aftik).unwrap();
            world
                .query::<With<item::Item, (&Pos, &DisplayInfo)>>()
                .iter()
                .filter(|(_, (pos, display_info))| {
                    pos.is_in(aftik_pos.get_area()) && display_info.matches(name)
                })
                .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
                .map(|(item, (_, display_info))| {
                    Ok(Some(Action::Wield(
                        item,
                        display_info.definite_name().to_string(),
                    )))
                })
        })
        .unwrap_or_else(|| {
            Err(format!(
                "There is no {} that {} can wield.",
                name,
                DisplayInfo::find_definite_name(world, aftik)
            ))
        })
    })
}

fn enter(parse: &Parse, world: &World, aftik: Entity) -> Result<Option<Action>, String> {
    parse.take_remaining(|name| {
        let area = world.get::<Pos>(aftik).unwrap().get_area();
        world
            .query::<With<door::Door, (&Pos, &DisplayInfo)>>()
            .iter()
            .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
            .map(|(door, _)| Ok(Some(Action::EnterDoor(door))))
            .unwrap_or_else(|| Err("There is no such door here to go through.".to_string()))
    })
}

fn force(parse: &Parse, world: &World, aftik: Entity) -> Result<Option<Action>, String> {
    parse.take_remaining(|name| {
        let area = world.get::<Pos>(aftik).unwrap().get_area();
        world
            .query::<With<door::Door, (&Pos, &DisplayInfo)>>()
            .iter()
            .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
            .map(|(door, _)| Ok(Some(Action::ForceDoor(door))))
            .unwrap_or_else(|| Err("There is no such door here.".to_string()))
    })
}

fn attack(parse: &Parse, world: &World, aftik: Entity) -> Result<Option<Action>, String> {
    parse.take_remaining(|name| {
        let area = world.get::<Pos>(aftik).unwrap().get_area();
        world
            .query::<With<combat::IsFoe, (&Pos, &DisplayInfo)>>()
            .iter()
            .find(|(_, (pos, display_info))| pos.is_in(area) && display_info.matches(name))
            .map(|(target, _)| Ok(Some(Action::Attack(target))))
            .unwrap_or_else(|| Err("There is no such target here.".to_string()))
    })
}

fn status(parse: &Parse, world: &World, aftik: Entity) -> Result<Option<Action>, String> {
    parse
        .done(|| {
            view::print_full_status(world, aftik);
            Ok(None)
        })
        .unwrap_or_else(|| Err("Unexpected argument after \"status\"".to_string()))
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

    fn take_remaining<F, T>(&self, closure: F) -> T
    where
        F: FnOnce(&str) -> T,
    {
        closure(self.input)
    }
}
