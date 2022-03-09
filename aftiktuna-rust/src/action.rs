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

fn enter_door(door: Entity, world: &mut World, game_state: &GameState) -> Result<String, String> {
    let destination = world
        .get::<Door>(door)
        .map_err(|_| "You lost track of the door.".to_string())?
        .destination;
    world.get_mut::<Position>(game_state.aftik).unwrap().0 = destination;

    Ok("You entered the door into a new location.".to_string())
}
