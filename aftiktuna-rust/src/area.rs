use crate::action::combat::{IsFoe, Weapon};
use crate::action::door::{Blowtorch, Crowbar, Door, Keycard};
use crate::action::item::{CanWield, FuelCan, Item};
use crate::action::Aftik;
use crate::position::{Coord, MovementBlocking, Pos};
use crate::status::{Health, Stats};
use crate::view::DisplayInfo;
use hecs::{DynamicBundle, Entity, World};

mod init;

pub fn init(world: &mut World) -> Entity {
    init::combat_test(world)
}

pub struct Area {
    pub size: Coord,
    pub label: String,
}

fn place_aftik(world: &mut World, area: Entity, coord: Coord, name: &str, stats: Stats) -> Entity {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_name(name.chars().next().unwrap(), name, 10),
        pos,
        Aftik,
        Health::with_max(&stats),
        stats,
    ))
}

fn place_goblin(world: &mut World, area: Entity, coord: Coord) -> Entity {
    let stats = Stats::new(2, 4, 10);
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('G', "goblin", 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        stats,
    ))
}

fn place_doors(
    world: &mut World,
    area1: Entity,
    coord1: Coord,
    disp1: DisplayInfo,
    area2: Entity,
    coord2: Coord,
    disp2: DisplayInfo,
    pair_components: impl DynamicBundle,
) {
    let pos1 = Pos::new(area1, coord1, world);
    let pos2 = Pos::new(area2, coord2, world);
    let door_pair = world.spawn(pair_components);
    place_door(world, pos1, disp1, pos2, door_pair);
    place_door(world, pos2, disp2, pos1, door_pair);
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

fn place_fuel(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('f', "fuel can", 1),
        pos,
        Item,
        FuelCan,
    ));
}

fn place_crowbar(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('c', "crowbar", 1),
        pos,
        Item,
        Crowbar,
        CanWield,
        Weapon(3.0),
    ));
}

fn place_blowtorch(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('b', "blowtorch", 1),
        pos,
        Item,
        Blowtorch,
    ));
}

fn place_keycard(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('k', "keycard", 1),
        pos,
        Item,
        Keycard,
    ));
}
