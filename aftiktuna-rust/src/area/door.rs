use crate::action::door::{BlockType, Door};
use crate::position::Pos;
use crate::view::{DisplayInfo, NameData, TextureType};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DoorInfo {
    pub pos: Pos,
    pub symbol: char,
    pub texture_type: TextureType,
    pub name: NameData,
}

pub fn place_pair(
    world: &mut World,
    door1: DoorInfo,
    door2: DoorInfo,
    block_type: Option<BlockType>,
) {
    let door_pair = match block_type {
        Some(block_type) => world.spawn((block_type,)),
        None => world.spawn(()),
    };
    let dest1 = door2.pos;
    let dest2 = door1.pos;
    place(world, door1, dest1, door_pair);
    place(world, door2, dest2, door_pair);
}

fn place(world: &mut World, info: DoorInfo, destination: Pos, door_pair: Entity) -> Entity {
    world.spawn((
        DisplayInfo::new(info.symbol, info.texture_type, 20),
        info.name,
        info.pos,
        Door {
            destination,
            door_pair,
        },
    ))
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    LeftDoor,
    MidDoor,
    RightDoor,
    Path,
    LeftPath,
    MidPath,
    RightPath,
}

impl DoorType {
    pub fn name_data(self) -> NameData {
        match self {
            DoorType::Door => NameData::from_noun("door", "doors"),
            DoorType::LeftDoor => NameData::from_noun("left door", "left doors"),
            DoorType::MidDoor => NameData::from_noun("middle door", "middle doors"),
            DoorType::RightDoor => NameData::from_noun("right door", "right doors"),
            DoorType::Path => NameData::from_noun("path", "paths"),
            DoorType::LeftPath => NameData::from_noun("left path", "left paths"),
            DoorType::MidPath => NameData::from_noun("middle path", "middle paths"),
            DoorType::RightPath => NameData::from_noun("right path", "right paths"),
        }
    }
    pub fn texture_type(self) -> TextureType {
        match self {
            DoorType::Door => TextureType::Door,
            DoorType::LeftDoor => TextureType::Door,
            DoorType::MidDoor => TextureType::Door,
            DoorType::RightDoor => TextureType::Door,
            DoorType::Path => TextureType::Path,
            DoorType::LeftPath => TextureType::Path,
            DoorType::MidPath => TextureType::Path,
            DoorType::RightPath => TextureType::Path,
        }
    }
}
