use crate::action::door::{BlockType, Door, DoorBlocking};
use crate::position::Pos;
use crate::view::{DisplayInfo, NameData};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DoorInfo(pub Pos, pub char, pub NameData);

pub fn place_pair(
    world: &mut World,
    door1: DoorInfo,
    door2: DoorInfo,
    block_type: Option<BlockType>,
) {
    let door_pair = match block_type {
        Some(block_type) => world.spawn((DoorBlocking(block_type),)),
        None => world.spawn(()),
    };
    place(world, door1.0, door1.1, door1.2, door2.0, door_pair);
    place(world, door2.0, door2.1, door2.2, door1.0, door_pair);
}

fn place(
    world: &mut World,
    pos: Pos,
    symbol: char,
    name: NameData,
    destination: Pos,
    door_pair: Entity,
) -> Entity {
    world.spawn((
        DisplayInfo::new(symbol, 20),
        name,
        pos,
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

pub fn name_data(door_type: DoorType) -> NameData {
    match door_type {
        DoorType::Door => NameData::from_noun("door"),
        DoorType::LeftDoor => NameData::from_noun("left door"),
        DoorType::MidDoor => NameData::from_noun("middle door"),
        DoorType::RightDoor => NameData::from_noun("right door"),
        DoorType::Path => NameData::from_noun("path"),
        DoorType::LeftPath => NameData::from_noun("left path"),
        DoorType::MidPath => NameData::from_noun("middle path"),
        DoorType::RightPath => NameData::from_noun("right path"),
    }
}
