use crate::area::{Coord, Position};
use crate::{GameState, Messages, Pos};
use hecs::{Entity, World};

#[derive(Debug, Default)]
pub struct FuelCan;

pub fn take_fuel_can(world: &mut World, game_state: &mut GameState, messages: &mut Messages) {
    let aftik = game_state.aftik.expect("Expected aftik to be initialized");
    let area = world.get::<Position>(aftik).unwrap().get_area();
    let option = find_fuel_can(area, world);

    match option {
        Some((fuel_can, item_pos)) => {
            world
                .get_mut::<Position>(aftik)
                .unwrap()
                .move_to(item_pos, world);
            world.despawn(fuel_can).unwrap();
            game_state.has_won = true;

            messages.0.push("You picked up the fuel can.".to_string());
        }
        None => {
            messages
                .0
                .push("There is no fuel can here to pick up.".to_string());
        }
    }
}

fn find_fuel_can(area: Entity, world: &World) -> Option<(Entity, Coord)> {
    world
        .query::<(&Position, &FuelCan)>()
        .iter()
        .filter(|(_, (pos, _))| pos.get_area().eq(&area))
        .next()
        .map(|(entity, (pos, _))| (entity, pos.get_coord()))
}

#[derive(Debug)]
pub struct Door {
    pub(crate) destination: Pos,
}

pub fn enter_door(world: &mut World, game_state: &GameState, messages: &mut Messages) {
    let aftik = game_state.aftik.expect("Expected aftik to be initialized");
    let area = world.get::<Position>(aftik).unwrap().get_area();
    let option = find_door(area, world);

    match option {
        Some((_, destination)) => {
            world.get_mut::<Position>(aftik).unwrap().0 = destination;

            messages
                .0
                .push("You entered the door into a new location.".to_string());
        }
        None => {
            messages
                .0
                .push("There is no door to go through.".to_string());
        }
    }
}

fn find_door(area: Entity, world: &World) -> Option<(Entity, Pos)> {
    world
        .query::<(&Position, &Door)>()
        .iter()
        .filter(|(_, (pos, _))| pos.get_area().eq(&area))
        .next()
        .map(|(entity, (_, door))| (entity, door.destination.clone()))
}
