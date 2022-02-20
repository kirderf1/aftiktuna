use crate::area::Position;
use crate::{DisplayInfo, GameState, Messages, Pos};
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
    let result = match action {
        TakeFuelCan(fuel_can) => take_fuel_can(fuel_can, world, game_state),
        EnterDoor(door) => enter_door(door, world, game_state),
    };
    match result {
        Ok(message) | Err(message) => messages.0.push(message),
    }
}

pub fn parse_take_fuel_can(world: &World, aftik: Entity) -> Result<Action, String> {
    let area = world.get::<Position>(aftik).unwrap().get_area();
    find_fuel_can(area, world).map(TakeFuelCan)
}

fn find_fuel_can(area: Entity, world: &World) -> Result<Entity, String> {
    world
        .query::<(&Position, &FuelCan)>()
        .iter()
        .find(|(_, (pos, _))| pos.get_area().eq(&area))
        .map(|(entity, _)| entity)
        .ok_or_else(|| "There is no fuel can here to pick up.".to_string())
}

fn take_fuel_can(
    fuel_can: Entity,
    world: &mut World,
    game_state: &mut GameState,
) -> Result<String, String> {
    let item_pos = world
        .get::<Position>(fuel_can)
        .map_err(|_| "You lost track of the fuel can.".to_string())?
        .get_coord();
    world
        .get_mut::<Position>(game_state.aftik)
        .unwrap()
        .move_to(item_pos, world);
    world.despawn(fuel_can).unwrap();
    game_state.has_won = true;

    Ok("You picked up the fuel can.".to_string())
}

#[derive(Debug)]
pub struct Door {
    pub destination: Pos,
}

pub fn parse_enter_door(world: &World, door_type: &str, aftik: Entity) -> Result<Action, String> {
    let area = world.get::<Position>(aftik).unwrap().get_area();
    find_door(area, door_type, world).map(EnterDoor)
}

fn find_door(area: Entity, door_type: &str, world: &World) -> Result<Entity, String> {
    world
        .query::<(&Position, &Door, &DisplayInfo)>()
        .iter()
        .find(|(_, (pos, _, disp))| {
            pos.get_area().eq(&area) && disp.name().eq_ignore_ascii_case(door_type)
        })
        .map(|(entity, _)| entity)
        .ok_or_else(|| "There is no such door to go through.".to_string())
}

fn enter_door(door: Entity, world: &mut World, game_state: &GameState) -> Result<String, String> {
    let destination = world
        .get::<Door>(door)
        .map_err(|_| "You lost track of the door.".to_string())?
        .destination;
    world.get_mut::<Position>(game_state.aftik).unwrap().0 = destination;

    Ok("You entered the door into a new location.".to_string())
}
