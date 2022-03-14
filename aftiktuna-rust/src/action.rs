use crate::{
    area::{Pos, Position},
    view::{DisplayInfo, Messages},
};
use hecs::{Component, Entity, World};
use Action::*;

pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    EnterDoor(Entity),
    ForceDoor(Entity),
}

fn try_move_aftik(world: &mut World, aftik: Entity, pos: Pos) -> Result<(), String> {
    let mut position = world.get_mut::<Position>(aftik).unwrap();
    assert_eq!(
        pos.get_area(),
        position.get_area(),
        "Areas should be equal when called."
    );

    position.0 = pos;
    Ok(())
}

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct InInventory;

pub fn has_item<C: Component>(world: &World) -> bool {
    world.query::<(&InInventory, &C)>().iter().len() > 0
}

pub fn run_action(world: &mut World, aftik: Entity, messages: &mut Messages) {
    if let Ok(action) = world.remove_one::<Action>(aftik) {
        let result = match action {
            TakeItem(item, name) => take_item(item, &name, world, aftik),
            TakeAll => take_all(world, aftik),
            EnterDoor(door) => enter_door(door, world, aftik),
            ForceDoor(door) => force_door(door, world, aftik),
        };
        match result {
            Ok(message) | Err(message) => messages.0.push(message),
        }
    }
}

fn take_all(world: &mut World, aftik: Entity) -> Result<String, String> {
    let area = world.get::<Position>(aftik).unwrap().get_area();
    let (item, name) = world
        .query::<(&Position, &DisplayInfo, &Item)>()
        .iter()
        .find(|(_, (pos, _, _))| pos.get_area().eq(&area))
        .map(|(item, (_, display_info, _))| (item, display_info.name().to_string()))
        .ok_or_else(|| "There are no items to take here.")?;

    let result = take_item(item, &name, world, aftik)?;
    if world
        .query::<(&Position, &DisplayInfo, &Item)>()
        .iter()
        .any(|(_, (pos, _, _))| pos.get_area().eq(&area))
    {
        world.insert_one(aftik, Action::TakeAll).unwrap();
    }
    Ok(result)
}

fn take_item(
    item: Entity,
    item_name: &str,
    world: &mut World,
    aftik: Entity,
) -> Result<String, String> {
    let item_pos = world
        .get::<Position>(item)
        .map_err(|_| format!("You lost track of the {}.", item_name))?
        .0;

    try_move_aftik(world, aftik, item_pos)?;
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
    pub door_pair: Entity,
}

#[derive(Debug)]
pub struct DoorBlocking(pub BlockType);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockType {
    Stuck,
    Sealed,
    Locked,
}

#[derive(Debug)]
pub struct Crowbar;

#[derive(Debug)]
pub struct Blowtorch;

#[derive(Debug)]
pub struct Keycard;

fn enter_door(door: Entity, world: &mut World, aftik: Entity) -> Result<String, String> {
    let area = world.get::<Position>(aftik).unwrap().0.get_area();
    let pos = world
        .get::<Position>(door)
        .ok()
        .filter(|pos| pos.0.get_area() == area)
        .ok_or_else(|| "You lost track of the door.".to_string())?
        .0;

    try_move_aftik(world, aftik, pos)?;

    let (destination, door_pair) = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| (door.destination, door.door_pair))?;

    let used_keycard = if let Ok(blocking) = world.get::<DoorBlocking>(door_pair) {
        if blocking.0 == BlockType::Locked && has_item::<Keycard>(world) {
            true
        } else {
            return Err(format!("The door is {}.", description(blocking.0)));
        }
    } else {
        false
    };

    world.get_mut::<Position>(aftik).unwrap().0 = destination;
    if used_keycard {
        Ok("Using your keycard, you entered the door into a new area.".to_string())
    } else {
        Ok("You entered the door into a new area.".to_string())
    }
}

pub fn description(t: BlockType) -> &'static str {
    match t {
        BlockType::Stuck => "stuck",
        BlockType::Sealed => "sealed shut",
        BlockType::Locked => "locked",
    }
}

fn force_door(door: Entity, world: &mut World, aftik: Entity) -> Result<String, String> {
    let area = world.get::<Position>(aftik).unwrap().0.get_area();
    let pos = world
        .get::<Position>(door)
        .ok()
        .filter(|pos| pos.0.get_area() == area)
        .ok_or_else(|| "You lost track of the door.".to_string())?
        .0;

    try_move_aftik(world, aftik, pos)?;

    let door_pair = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())?
        .door_pair;

    let block_type = world
        .get::<DoorBlocking>(door_pair)
        .map(|blocking| blocking.0);
    if let Ok(block_type) = block_type {
        match block_type {
            BlockType::Stuck => {
                if has_item::<Crowbar>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok("You used your crowbar and forced open the door.".to_string())
                } else if has_item::<Blowtorch>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok("You used your blowtorch and cut open the door.".to_string())
                } else {
                    Err("You need some sort of tool to force the door open.".to_string())
                }
            }
            BlockType::Sealed => {
                if has_item::<Blowtorch>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok("You used your blowtorch and cut open the door.".to_string())
                } else {
                    Err("You need some sort of tool to break the door open.".to_string())
                }
            }
            BlockType::Locked => {
                if has_item::<Blowtorch>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok("You used your blowtorch and cut open the door.".to_string())
                } else {
                    Err("You need some sort of tool to break the door open.".to_string())
                }
            }
        }
    } else {
        Err("The door does not seem to be stuck.".to_string())
    }
}
