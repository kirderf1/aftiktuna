use crate::view::DisplayInfo;
use hecs::{Component, Entity, World};
use serde::{Deserialize, Serialize};

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

// A type handy for spawning a variable type of item
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    FuelCan,
    Crowbar,
    Blowtorch,
    Keycard,
    Knife,
    Bat,
    Sword,
    MeteorChunk,
    AncientCoin,
}

pub fn spawn(world: &mut World, item_type: Type, location: impl Component) -> Entity {
    match item_type {
        Type::FuelCan => spawn_fuel_can(world, location),
        Type::Crowbar => spawn_crowbar(world, location),
        Type::Blowtorch => spawn_blowtorch(world, location),
        Type::Keycard => spawn_keycard(world, location),
        Type::Knife => spawn_knife(world, location),
        Type::Bat => spawn_bat(world, location),
        Type::Sword => spawn_sword(world, location),
        Type::MeteorChunk => spawn_meteor_chunk(world, location),
        Type::AncientCoin => spawn_ancient_coin(world, location),
    }
}

pub fn spawn_fuel_can(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('f', "fuel can", 1),
        Item,
        FuelCan,
    ))
}

pub fn spawn_crowbar(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('c', "crowbar", 1),
        Item,
        Crowbar,
        CanWield,
        Weapon(3.0),
    ))
}

pub fn spawn_blowtorch(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('b', "blowtorch", 1),
        Item,
        Blowtorch,
    ))
}

pub fn spawn_keycard(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('k', "keycard", 1),
        Item,
        Keycard,
    ))
}

pub fn spawn_knife(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('K', "knife", 1),
        Item,
        CanWield,
        Weapon(3.0),
    ))
}

pub fn spawn_bat(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('B', "bat", 1),
        Item,
        CanWield,
        Weapon(4.0),
    ))
}

pub fn spawn_sword(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('s', "sword", 1),
        Item,
        CanWield,
        Weapon(5.0),
    ))
}

pub fn spawn_meteor_chunk(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('m', "meteor chunk", 1),
        Item,
    ))
}

pub fn spawn_ancient_coin(world: &mut World, location: impl Component) -> Entity {
    world.spawn((
        location,
        DisplayInfo::from_noun('a', "ancient coin", 1),
        Item,
    ))
}
