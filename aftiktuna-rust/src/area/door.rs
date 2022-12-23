use crate::action::door::{BlockType, Door, DoorBlocking};
use crate::position::Pos;
use crate::view::DisplayInfo;
use hecs::{Entity, World};

#[derive(Clone)]
pub struct DoorInfo(pub Pos, pub DisplayInfo);

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
    place(world, door1.0, door1.1, door2.0, door_pair);
    place(world, door2.0, door2.1, door1.0, door_pair);
}

fn place(
    world: &mut World,
    pos: Pos,
    disp: DisplayInfo,
    destination: Pos,
    door_pair: Entity,
) -> Entity {
    world.spawn((
        disp,
        pos,
        Door {
            destination,
            door_pair,
        },
    ))
}

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

pub fn door_display(door_type: &DoorType) -> DisplayInfo {
    match door_type {
        DoorType::Door => DisplayInfo::from_noun('^', "door", 20),
        DoorType::LeftDoor => DisplayInfo::from_noun('<', "left door", 20),
        DoorType::MidDoor => DisplayInfo::from_noun('^', "middle door", 20),
        DoorType::RightDoor => DisplayInfo::from_noun('>', "right door", 20),
        DoorType::Path => DisplayInfo::from_noun('^', "path", 20),
        DoorType::LeftPath => DisplayInfo::from_noun('<', "left path", 20),
        DoorType::MidPath => DisplayInfo::from_noun('^', "middle path", 20),
        DoorType::RightPath => DisplayInfo::from_noun('>', "right path", 20),
    }
}
