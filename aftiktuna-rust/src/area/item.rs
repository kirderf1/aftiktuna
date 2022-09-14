use hecs::{Entity, World};
use crate::action::combat::Weapon;
use crate::action::door::{Blowtorch, Crowbar, Keycard};
use crate::action::item::{CanWield, FuelCan, Item};
use crate::position::{Coord, Pos};
use crate::view::DisplayInfo;

pub fn place_fuel(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('f', "fuel can", 1),
        pos,
        Item,
        FuelCan,
    ));
}

pub fn place_crowbar(world: &mut World, area: Entity, coord: Coord) {
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

pub fn place_blowtorch(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('b', "blowtorch", 1),
        pos,
        Item,
        Blowtorch,
    ));
}

pub fn place_keycard(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('k', "keycard", 1),
        pos,
        Item,
        Keycard,
    ));
}

pub fn place_knife(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('K', "knife", 1),
        pos,
        Item,
        CanWield,
        Weapon(3.0),
    ));
}

pub fn place_bat(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('B', "bat", 1),
        pos,
        Item,
        CanWield,
        Weapon(4.0),
    ));
}

pub fn place_sword(world: &mut World, area: Entity, coord: Coord) {
    let pos = Pos::new(area, coord, world);
    world.spawn((
        DisplayInfo::from_noun('s', "sword", 1),
        pos,
        Item,
        CanWield,
        Weapon(5.0),
    ));
}
