use crate::action::door::{BlockType, Door, DoorBlocking};
use crate::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::DisplayInfo;
use hecs::{Entity, World};

mod creature;
mod init;
mod item;
mod template;

pub fn init(world: &mut World) -> Entity {
    let start_pos = init::combat_test().build(world);
    let ship = world.spawn((
        Area {
            label: "Ship".to_string(),
            size: 4,
        },
        Ship(ShipStatus::NeedTwoCans),
    ));
    place_doors(
        world,
        DoorInfo(start_pos, DisplayInfo::from_noun('v', "ship entrance", 20)),
        DoorInfo(
            Pos::new(ship, 3, world),
            DisplayInfo::from_noun('^', "ship exit", 20),
        ),
        None,
    );

    creature::place_aftik(world, start_pos, "Cerulean", Stats::new(9, 2, 10));
    creature::place_aftik(world, start_pos, "Mint", Stats::new(10, 3, 8))
}

pub struct Area {
    pub size: Coord,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct Ship(pub ShipStatus);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShipStatus {
    NeedTwoCans,
    NeedOneCan,
    Launching,
}

#[derive(Clone)]
struct DoorInfo(Pos, DisplayInfo);

fn place_doors(world: &mut World, door1: DoorInfo, door2: DoorInfo, block_type: Option<BlockType>) {
    let door_pair = match block_type {
        Some(block_type) => world.spawn((DoorBlocking(block_type),)),
        None => world.spawn(()),
    };
    place_door(world, door1.0, door1.1, door2.0, door_pair);
    place_door(world, door2.0, door2.1, door1.0, door_pair);
}

fn place_door(
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

fn door() -> DisplayInfo {
    DisplayInfo::from_noun('^', "door", 20)
}

fn left_door() -> DisplayInfo {
    DisplayInfo::from_noun('<', "left door", 20)
}

fn right_door() -> DisplayInfo {
    DisplayInfo::from_noun('>', "right door", 20)
}
