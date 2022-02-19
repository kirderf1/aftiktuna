use crate::area::Position;
use crate::{GameState, Messages, Pos};
use hecs::{Entity, World};
use Action::*;

pub enum Action {
    TakeFuelCan(Entity),
    EnterDoor(Entity),
}

#[derive(Debug, Default)]
pub struct FuelCan;

pub fn run_action(
    action: Action,
    world: &mut World,
    game_state: &mut GameState,
    messages: &mut Messages,
) {
    match action {
        TakeFuelCan(fuel_can) => take_fuel_can(fuel_can, world, game_state, messages),
        EnterDoor(door) => enter_door(door, world, game_state, messages),
    }
}

pub fn parse_take_fuel_can(world: &World, game_state: &GameState) -> Result<Action, String> {
    let area = world.get::<Position>(game_state.aftik).unwrap().get_area();
    let option = find_fuel_can(area, world);

    match option {
        Some(fuel_can) => Ok(TakeFuelCan(fuel_can)),
        None => {
            Err("There is no fuel can here to pick up.".to_string())
        }
    }
}

fn take_fuel_can(
    fuel_can: Entity,
    world: &mut World,
    game_state: &mut GameState,
    messages: &mut Messages,
) {
    let item_pos = world.get::<Position>(fuel_can).unwrap().get_coord();
    world
        .get_mut::<Position>(game_state.aftik)
        .unwrap()
        .move_to(item_pos, world);
    world.despawn(fuel_can).unwrap();
    game_state.has_won = true;

    messages.0.push("You picked up the fuel can.".to_string());
}

fn find_fuel_can(area: Entity, world: &World) -> Option<Entity> {
    world
        .query::<(&Position, &FuelCan)>()
        .iter()
        .filter(|(_, (pos, _))| pos.get_area().eq(&area))
        .next()
        .map(|(entity, _)| entity)
}

#[derive(Debug)]
pub struct Door {
    pub destination: Pos,
}

pub fn parse_enter_door(world: &World, game_state: &GameState) -> Result<Action, String> {
    let area = world.get::<Position>(game_state.aftik).unwrap().get_area();
    let option = find_door(area, world);

    match option {
        Some(door) => Ok(EnterDoor(door)),
        None => {
            Err("There is no door to go through.".to_string())
        }
    }
}

fn enter_door(door: Entity, world: &mut World, game_state: &GameState, messages: &mut Messages) {
    let destination = world.get::<Door>(door).unwrap().destination.clone();
    world.get_mut::<Position>(game_state.aftik).unwrap().0 = destination;

    messages
        .0
        .push("You entered the door into a new location.".to_string());
}

fn find_door(area: Entity, world: &World) -> Option<Entity> {
    world
        .query::<(&Position, &Door)>()
        .iter()
        .filter(|(_, (pos, _))| pos.get_area().eq(&area))
        .next()
        .map(|(entity, _)| entity)
}
