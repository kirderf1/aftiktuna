use crate::{
    area::{Pos, Position},
    view::{DisplayInfo, Messages},
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
        TakeItem(item, name) => take_item(item, name, world, game_state),
        EnterDoor(door) => enter_door(door, world, game_state),
    };
    match result {
        Ok(message) | Err(message) => messages.0.push(message),
    }
}

pub fn parse_take_item(world: &World, item_name: &str, aftik: Entity) -> Result<Action, String> {
    let area = world.get::<Position>(aftik).unwrap().get_area();
    find_item(area, item_name, world).map(|item| TakeItem(item, item_name.to_string()))
}

fn find_item(area: Entity, item_type: &str, world: &World) -> Result<Entity, String> {
    world
        .query::<(&Position, &Item, &DisplayInfo)>()
        .iter()
        .find(|(_, (pos, _, disp))| {
            pos.get_area().eq(&area) && disp.name().eq_ignore_ascii_case(item_type)
        })
        .map(|(entity, _)| entity)
        .ok_or_else(|| "There is no fuel can here to pick up.".to_string())
}

fn take_item(
    item: Entity,
    item_name: String,
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
