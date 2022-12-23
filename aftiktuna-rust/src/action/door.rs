use crate::action::item;
use crate::position;
use crate::position::Pos;
use crate::view::DisplayInfo;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Door {
    pub destination: Pos,
    pub door_pair: Entity,
}

#[derive(Debug)]
pub struct DoorBlocking(pub BlockType);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Stuck,
    Sealed,
    Locked,
}

impl BlockType {
    pub fn description(self) -> &'static str {
        match self {
            BlockType::Stuck => "stuck",
            BlockType::Sealed => "sealed shut",
            BlockType::Locked => "locked",
        }
    }

    fn try_force(
        self,
        world: &mut World,
        aftik: Entity,
        aftik_name: String,
    ) -> Result<String, String> {
        match self {
            BlockType::Stuck => {
                if item::is_holding::<Crowbar>(world, aftik) {
                    Ok(format!(
                        "{} used their crowbar and forced open the door.",
                        aftik_name
                    ))
                } else if item::is_holding::<Blowtorch>(world, aftik) {
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
                if item::is_holding::<Blowtorch>(world, aftik) {
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
    let door_pos = *world
        .get::<Pos>(door)
        .ok()
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;
    if Ok(door_pos.get_area()) != world.get::<Pos>(aftik).map(|pos| pos.get_area()) {
        return Err(format!("{} cannot reach the door from here.", aftik_name));
    }

    position::try_move(world, aftik, door_pos)?;

    let (destination, door_pair) = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| (door.destination, door.door_pair))?;

    let used_keycard = if let Ok(blocking) = world.get::<DoorBlocking>(door_pair) {
        if blocking.0 == BlockType::Locked && item::is_holding::<Keycard>(world, aftik) {
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
    let door_pos = *world
        .get::<Pos>(door)
        .ok()
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;
    if Ok(door_pos.get_area()) != world.get::<Pos>(aftik).map(|pos| pos.get_area()) {
        return Err(format!("{} cannot reach the door from here.", aftik_name));
    }

    position::try_move(world, aftik, door_pos)?;

    let door_pair = world
        .get::<Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())?
        .door_pair;

    let block_type = world
        .get::<DoorBlocking>(door_pair)
        .map(|blocking| blocking.0);
    if let Ok(block_type) = block_type {
        let result = block_type.try_force(world, aftik, aftik_name);
        if result.is_ok() {
            world.remove_one::<DoorBlocking>(door_pair).unwrap();
        }
        result
    } else {
        Err("The door does not seem to be stuck.".to_string())
    }
}
