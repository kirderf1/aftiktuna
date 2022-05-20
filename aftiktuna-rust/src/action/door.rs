use crate::action::item;
use crate::position;
use crate::position::Pos;
use crate::view::DisplayInfo;
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

impl BlockType {
    pub fn description(&self) -> &'static str {
        match self {
            BlockType::Stuck => "stuck",
            BlockType::Sealed => "sealed shut",
            BlockType::Locked => "locked",
        }
    }

}

#[derive(Debug)]
pub struct Crowbar;

#[derive(Debug)]
pub struct Blowtorch;

#[derive(Debug)]
pub struct Keycard;

pub fn enter_door(world: &mut World, aftik: Entity, door: Entity) -> Result<String, String> {
    let aftik_name = DisplayInfo::find_definite_name(world, aftik);
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    let pos = *world
        .get::<Pos>(door)
        .ok()
        .filter(|pos| pos.get_area() == area)
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;

    position::try_move(world, aftik, pos)?;

    let (destination, door_pair) = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| (door.destination, door.door_pair))?;

    let used_keycard = if let Ok(blocking) = world.get::<DoorBlocking>(door_pair) {
        if blocking.0 == BlockType::Locked && item::has_item::<Keycard>(world) {
            true
        } else {
            return Err(format!("The door is {}.", blocking.0.description()));
        }
    } else {
        false
    };

    world.insert_one(aftik, destination).unwrap();
    if used_keycard {
        Ok(format!(
            "Using their keycard, {} entered the door into a new area.",
            aftik_name
        ))
    } else {
        Ok(format!("{} entered the door into a new area.", aftik_name))
    }
}

pub fn force_door(world: &mut World, aftik: Entity, door: Entity) -> Result<String, String> {
    let aftik_name = DisplayInfo::find_definite_name(world, aftik);
    let area = world.get::<Pos>(aftik).unwrap().get_area();
    let pos = *world
        .get::<Pos>(door)
        .ok()
        .filter(|pos| pos.get_area() == area)
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;

    position::try_move(world, aftik, pos)?;

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
                    Ok(format!(
                        "{} used their crowbar and forced open the door.",
                        aftik_name
                    ))
                } else if item::has_item::<Blowtorch>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok(format!(
                        "{} used their blowtorch and cut open the door.",
                        aftik_name
                    ))
                } else {
                    Err(format!(
                        "{} needs some sort of tool to force the door open.",
                        aftik_name
                    ))
                }
            }
            BlockType::Sealed | BlockType::Locked => {
                if item::has_item::<Blowtorch>(world) {
                    world.remove_one::<DoorBlocking>(door_pair).unwrap();
                    Ok(format!(
                        "{} used their blowtorch and cut open the door.",
                        aftik_name
                    ))
                } else {
                    Err(format!(
                        "{} needs some sort of tool to break the door open.",
                        aftik_name
                    ))
                }
            }
        }
    } else {
        Err("The door does not seem to be stuck.".to_string())
    }
}
