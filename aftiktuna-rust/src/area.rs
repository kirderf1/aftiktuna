use crate::action::door::Door;
use crate::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::DisplayInfo;
use hecs::{DynamicBundle, Entity, World};

mod creature;
mod init;
mod item;

pub fn init(world: &mut World) -> Entity {
    let (start_area, start_coord) = init::combat_test(world);
    let ship = world.spawn((
        Area {
            label: "Ship".to_string(),
            size: 4,
        },
        Ship(ShipStatus::NeedTwoCans),
    ));
    place_doors(
        world,
        DoorInfo(
            start_area,
            start_coord,
            DisplayInfo::from_noun('v', "ship entrance", 20),
        ),
        DoorInfo(ship, 3, DisplayInfo::from_noun('^', "ship exit", 20)),
        (),
    );

    creature::place_aftik(
        world,
        start_area,
        start_coord,
        "Cerulean",
        Stats::new(9, 2, 10),
    );
    creature::place_aftik(world, start_area, start_coord, "Mint", Stats::new(10, 3, 8))
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

struct DoorInfo(Entity, Coord, DisplayInfo);

fn place_doors(
    world: &mut World,
    door1: DoorInfo,
    door2: DoorInfo,
    pair_components: impl DynamicBundle,
) {
    let pos1 = Pos::new(door1.0, door1.1, world);
    let pos2 = Pos::new(door2.0, door2.1, world);
    let door_pair = world.spawn(pair_components);
    place_door(world, pos1, door1.2, pos2, door_pair);
    place_door(world, pos2, door2.2, pos1, door_pair);
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
