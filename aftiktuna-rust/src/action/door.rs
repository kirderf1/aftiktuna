use crate::action;
use crate::core::item::{Keycard, Tool};
use crate::core::position::Pos;
use crate::core::{inventory, position};
use crate::view::{DisplayInfo, NameData, TextureType};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Door {
    pub kind: DoorKind,
    pub destination: Pos,
    pub door_pair: Entity,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DoorKind {
    Door,
    Path,
}

impl DoorKind {
    fn get_move_message(self, performer: &str) -> String {
        match self {
            DoorKind::Door => format!("{} entered the door into a new area.", performer),
            DoorKind::Path => format!("{} followed the path to a new area.", performer),
        }
    }
}

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

    fn try_force(self, world: &mut World, aftik: Entity, aftik_name: &str) -> Result<Tool, String> {
        match self {
            BlockType::Stuck => {
                if inventory::is_holding_tool(world, aftik, Tool::Crowbar) {
                    Ok(Tool::Crowbar)
                } else if inventory::is_holding_tool(world, aftik, Tool::Blowtorch) {
                    Ok(Tool::Blowtorch)
                } else {
                    Err(format!(
                        "{} needs some sort of tool to force the door open.",
                        aftik_name
                    ))
                }
            }
            BlockType::Sealed | BlockType::Locked => {
                if inventory::is_holding_tool(world, aftik, Tool::Blowtorch) {
                    Ok(Tool::Blowtorch)
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

pub fn enter_door(world: &mut World, aftik: Entity, door: Entity) -> action::Result {
    let aftik_name = NameData::find(world, aftik).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(aftik).map(|pos| pos.get_area()) {
        return Err(format!("{} cannot reach the door from here.", aftik_name));
    }

    position::try_move(world, aftik, door_pos)?;

    let door = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| door.deref().clone())?;

    let used_keycard = if let Ok(blocking) = world.get::<&BlockType>(door.door_pair) {
        if *blocking == BlockType::Locked && inventory::is_holding::<&Keycard>(world, aftik) {
            true
        } else {
            return Err(format!("The door is {}.", blocking.description()));
        }
    } else {
        false
    };

    world.insert_one(aftik, door.destination).unwrap();
    let areas = vec![door_pos.get_area(), door.destination.get_area()];
    if used_keycard {
        action::ok_at(
            format!(
                "Using their keycard, {}",
                door.kind.get_move_message(&aftik_name),
            ),
            areas,
        )
    } else {
        action::ok_at(door.kind.get_move_message(&aftik_name), areas)
    }
}

pub fn force_door(world: &mut World, aftik: Entity, door: Entity) -> action::Result {
    let aftik_name = NameData::find(world, aftik).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{} lost track of the door.", aftik_name))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(aftik).map(|pos| pos.get_area()) {
        return Err(format!("{} cannot reach the door from here.", aftik_name));
    }

    position::try_move(world, aftik, door_pos)?;

    let door_pair = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())?
        .door_pair;

    let block_type = *world
        .get::<&BlockType>(door_pair)
        .map_err(|_| "The door does not seem to be stuck.".to_string())?;

    let tool = block_type.try_force(world, aftik, &aftik_name)?;
    world.remove_one::<BlockType>(door_pair).unwrap();
    if tool == Tool::Blowtorch {
        world
            .query::<(&Door, &mut DisplayInfo)>()
            .iter()
            .filter(|(_, (door, _))| door.door_pair == door_pair)
            .for_each(|(_, (_, display_info))| set_is_cut(display_info));
    }

    action::ok(tool.into_message(&aftik_name))
}

fn set_is_cut(display_info: &mut DisplayInfo) {
    if display_info.texture_type == TextureType::Door {
        display_info.texture_type = TextureType::CutDoor;
    } else if display_info.texture_type == TextureType::Shack {
        display_info.texture_type = TextureType::CutShack;
    }
}
