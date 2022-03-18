use crate::action::item;
use crate::position;
use crate::position::{Pos, Position};
use hecs::{Entity, World};

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

pub fn enter_door(world: &mut World, aftik: Entity, door: Entity) -> Result<String, String> {
    let area = world.get::<Position>(aftik).unwrap().0.get_area();
    let pos = world
        .get::<Position>(door)
        .ok()
        .filter(|pos| pos.0.get_area() == area)
        .ok_or_else(|| "You lost track of the door.".to_string())?
        .0;

    position::try_move_aftik(world, aftik, pos)?;

    let (destination, door_pair) = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| (door.destination, door.door_pair))?;

    let used_keycard = if let Ok(blocking) = world.get::<DoorBlocking>(door_pair) {
        if blocking.0 == BlockType::Locked && item::has_item::<Keycard>(world) {
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

pub fn force_door(world: &mut World, aftik: Entity, door: Entity) -> Result<String, String> {
    let area = world.get::<Position>(aftik).unwrap().0.get_area();
    let pos = world
        .get::<Position>(door)
        .ok()
        .filter(|pos| pos.0.get_area() == area)
        .ok_or_else(|| "You lost track of the door.".to_string())?
        .0;

    position::try_move_aftik(world, aftik, pos)?;

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
                if item::has_item::<Crowbar>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok("You used your crowbar and forced open the door.".to_string())
                } else if item::has_item::<Blowtorch>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok("You used your blowtorch and cut open the door.".to_string())
                } else {
                    Err("You need some sort of tool to force the door open.".to_string())
                }
            }
            BlockType::Sealed | BlockType::Locked => {
                if item::has_item::<Blowtorch>(world) {
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
