use crate::view::DisplayInfo;
use hecs::{Component, World};

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct Crowbar;

#[derive(Debug)]
pub struct Blowtorch;

#[derive(Debug)]
pub struct Keycard;

#[derive(Debug, Default)]
pub struct CanWield;

#[derive(Debug)]
pub struct Weapon(pub f32);

pub fn spawn_fuel_can(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('f', "fuel can", 1),
        Item,
        FuelCan,
    ));
}

pub fn spawn_crowbar(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('c', "crowbar", 1),
        Item,
        Crowbar,
        CanWield,
        Weapon(3.0),
    ));
}

pub fn spawn_blowtorch(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('b', "blowtorch", 1),
        Item,
        Blowtorch,
    ));
}

pub fn spawn_keycard(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('k', "keycard", 1),
        Item,
        Keycard,
    ));
}

pub fn spawn_knife(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('K', "knife", 1),
        Item,
        CanWield,
        Weapon(3.0),
    ));
}

pub fn spawn_bat(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('B', "bat", 1),
        Item,
        CanWield,
        Weapon(4.0),
    ));
}

pub fn spawn_sword(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('s', "sword", 1),
        Item,
        CanWield,
        Weapon(5.0),
    ));
}

pub fn spawn_meteor_chunk(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('m', "meteor chunk", 1),
        Item,
    ));
}

pub fn spawn_ancient_coin(world: &mut World, location: impl Component) {
    world.spawn((
        location,
        DisplayInfo::from_noun('a', "ancient coin", 1),
        Item,
    ));
}
