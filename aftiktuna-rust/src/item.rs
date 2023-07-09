use crate::view::{DisplayInfo, NameData, NounData, TextureType};
use hecs::{Component, Entity, EntityBuilder, World};
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

pub struct Medkit;

#[derive(Debug, Default)]
pub struct CanWield;

#[derive(Debug)]
pub struct Weapon(pub f32);

pub struct Price(pub i32);

// A type handy for spawning a variable type of item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    FuelCan,
    Crowbar,
    Blowtorch,
    Keycard,
    Knife,
    Bat,
    Sword,
    Medkit,
    MeteorChunk,
    AncientCoin,
}

impl Type {
    pub fn spawn(self, world: &mut World, location: impl Component) {
        spawn(world, self, location);
    }

    pub fn noun_data(self) -> NounData {
        match self {
            Type::FuelCan => NounData::new("fuel can", "fuel cans"),
            Type::Crowbar => NounData::new("crowbar", "crowbars"),
            Type::Blowtorch => NounData::new("blowtorch", "blowtorches"),
            Type::Keycard => NounData::new("keycard", "keycards"),
            Type::Knife => NounData::new("knife", "knives"),
            Type::Bat => NounData::new("bat", "bats"),
            Type::Sword => NounData::new("sword", "swords"),
            Type::Medkit => NounData::new("medkit", "medkits"),
            Type::MeteorChunk => NounData::new("meteor chunk", "meteor chunks"),
            Type::AncientCoin => NounData::new("ancient coin", "ancient coins"),
        }
    }

    pub fn symbol(self) -> char {
        match self {
            Type::FuelCan => 'f',
            Type::Crowbar => 'c',
            Type::Blowtorch => 'b',
            Type::Keycard => 'k',
            Type::Knife => 'K',
            Type::Bat => 'B',
            Type::Sword => 's',
            Type::Medkit => '+',
            Type::MeteorChunk => 'm',
            Type::AncientCoin => 'a',
        }
    }

    pub fn display_info(self) -> DisplayInfo {
        DisplayInfo::new(self.symbol(), TextureType::Item(self), 1)
    }

    pub fn price(self) -> Option<i32> {
        match self {
            Type::FuelCan => Some(3500),
            Type::Knife => Some(300),
            Type::Bat => Some(1000),
            Type::Sword => Some(3000),
            Type::MeteorChunk => Some(2500),
            Type::AncientCoin => Some(500),
            _ => None,
        }
    }
}

pub fn spawn(world: &mut World, item_type: Type, location: impl Component) -> Entity {
    let mut builder = EntityBuilder::new();
    builder
        .add(location)
        .add(Item)
        .add(item_type.display_info())
        .add(NameData::Noun(item_type.noun_data()));
    if let Some(price) = item_type.price() {
        builder.add(Price(price));
    }

    match item_type {
        Type::FuelCan => {
            builder.add(FuelCan);
        }
        Type::Crowbar => {
            builder.add(Crowbar).add(CanWield).add(Weapon(3.0));
        }
        Type::Blowtorch => {
            builder.add(Blowtorch);
        }
        Type::Keycard => {
            builder.add(Keycard);
        }
        Type::Knife => {
            builder.add(CanWield).add(Weapon(3.0));
        }
        Type::Bat => {
            builder.add(CanWield).add(Weapon(4.0));
        }
        Type::Sword => {
            builder.add(CanWield).add(Weapon(5.0));
        }
        Type::Medkit => {
            builder.add(Medkit);
        }
        _ => {}
    };
    world.spawn(builder.build())
}
