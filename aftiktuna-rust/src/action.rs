use crate::{
    area::{Pos, Position},
    view::Messages,
    GameState,
};
use hecs::{Entity, World};
use Action::*;

pub enum Action {
    TakeItem(Entity, String),
    EnterDoor(Entity),
    ForceDoor(Entity),
}

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct InInventory;

pub fn has_fuel_can(world: &World) -> bool {
    world.query::<(&InInventory, &FuelCan)>().iter().len() > 0
}

pub fn run_action(
    action: Action,
    world: &mut World,
    game_state: &GameState,
    messages: &mut Messages,
) {
    let result = match action {
        TakeItem(item, name) => take_item(item, &name, world, game_state),
        EnterDoor(door) => enter_door(door, world, game_state),
        ForceDoor(door) => force_door(door, world, game_state),
    };
    match result {
        Ok(message) | Err(message) => messages.0.push(message),
    }
}

fn take_item(
    item: Entity,
    item_name: &str,
    world: &mut World,
    game_state: &GameState,
) -> Result<String, String> {
    let item_pos = world
        .get::<Position>(item)
        .map_err(|_| format!("You lost track of the {}.", item_name))?
        .get_coord();
    world
        .get_mut::<Position>(game_state.aftik)
        .unwrap()
        .move_to(item_pos, world);
    world
        .remove_one::<Position>(item)
        .expect("Tried removing position from item");
    world
        .insert_one(item, InInventory)
        .expect("Tried adding inventory data to item");

    Ok(format!("You picked up the {}.", item_name))
}

#[derive(Debug)]
pub struct Door {
    pub destination: Pos,
}

#[derive(Debug)]
pub struct IsStuck;

#[derive(Debug)]
pub struct Crowbar;

fn enter_door(door: Entity, world: &mut World, game_state: &GameState) -> Result<String, String> {
    let area = world
        .get::<Position>(game_state.aftik)
        .unwrap()
        .0
        .get_area();
    let pos = world
        .get::<Position>(door)
        .ok()
        .filter(|pos| pos.0.get_area() == area)
        .ok_or_else(|| "You lost track of the door.".to_string())?
        .0;

    world.get_mut::<Position>(game_state.aftik).unwrap().0 = pos;

    if world.get::<IsStuck>(door).is_ok() {
        return Err("The door is stuck.".to_string());
    }

    let destination = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())?
        .destination;

    world.get_mut::<Position>(game_state.aftik).unwrap().0 = destination;
    Ok("You entered the door into a new location.".to_string())
}

fn force_door(door: Entity, world: &mut World, game_state: &GameState) -> Result<String, String> {
    let area = world
        .get::<Position>(game_state.aftik)
        .unwrap()
        .0
        .get_area();
    let pos = world
        .get::<Position>(door)
        .ok()
        .filter(|pos| pos.0.get_area() == area)
        .ok_or_else(|| "You lost track of the door.".to_string())?
        .0;

    world.get_mut::<Position>(game_state.aftik).unwrap().0 = pos;

    if world.get::<IsStuck>(door).is_err() {
        return Err("The door does not seem to be stuck.".to_string());
    }

    if !has_crowbar(world) {
        return Err("You need some sort of tool to force the door open.".to_string());
    }

    world.remove_one::<IsStuck>(door).unwrap();
    Ok("You used your crowbar and forced open the door.".to_string())
}

fn has_crowbar(world: &World) -> bool {
    world.query::<(&InInventory, &Crowbar)>().iter().len() > 0
}
