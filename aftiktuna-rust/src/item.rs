use crate::view::{DisplayInfo, NameData};
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

#[derive(Debug, Default)]
pub struct CanWield;

#[derive(Debug)]
pub struct Weapon(pub f32);

pub struct Price(pub i32);

// A type handy for spawning a variable type of item
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

impl Type {
    pub fn spawn(self, world: &mut World, location: impl Component) {
        spawn(world, self, location);
    }

    pub fn name_for_amount(self, amount: i32) -> String {
        if amount == 1 {
            self.name_data().base().to_string()
        } else {
            match self {
                Type::FuelCan => "fuel cans",
                Type::Crowbar => "crowbars",
                Type::Blowtorch => "blowtorches",
                Type::Keycard => "keycards",
                Type::Knife => "knives",
                Type::Bat => "bats",
                Type::Sword => "swords",
                Type::MeteorChunk => "meteor chunks",
                Type::AncientCoin => "ancient coins",
            }
            .to_string()
        }
    }

    pub fn name_data(self) -> NameData {
        match self {
            Type::FuelCan => NameData::from_noun("fuel can"),
            Type::Crowbar => NameData::from_noun("crowbar"),
            Type::Blowtorch => NameData::from_noun("blowtorch"),
            Type::Keycard => NameData::from_noun("keycard"),
            Type::Knife => NameData::from_noun("knife"),
            Type::Bat => NameData::from_noun("bat"),
            Type::Sword => NameData::from_noun("sword"),
            Type::MeteorChunk => NameData::from_noun("meteor chunk"),
            Type::AncientCoin => NameData::from_noun("ancient coin"),
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
            Type::MeteorChunk => 'm',
            Type::AncientCoin => 'a',
        }
    }

    pub fn display_info(self) -> DisplayInfo {
        DisplayInfo::new(self.symbol(), self.name_data(), 1)
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
        .add(item_type.display_info());
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
        _ => {}
    };
    world.spawn(builder.build())
}
