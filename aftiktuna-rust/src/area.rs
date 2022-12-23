use crate::action::door::{BlockType, Door, DoorBlocking};
use crate::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::DisplayInfo;
use door::DoorInfo;
use hecs::{Entity, World};

mod creature;
mod door;
mod init;
mod item;
mod template;

pub fn init(world: &mut World) -> Entity {
    let start_pos = init::abandoned_facility().build(world);
    let ship = world.spawn((
        Area {
            label: "Ship".to_string(),
            size: 4,
        },
        Ship(ShipStatus::NeedTwoCans),
    ));
    door::place_pair(
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
