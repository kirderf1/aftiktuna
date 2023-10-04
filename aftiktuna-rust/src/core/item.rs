use crate::view::name::Noun;
use crate::view::{OrderWeight, Symbol, TextureType};
use hecs::{Component, Entity, EntityBuilder, World};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Item;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FuelCan;

#[derive(Serialize, Deserialize)]
pub struct FoodRation;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Tool {
    Crowbar,
    Blowtorch,
}

impl Tool {
    pub fn into_message(self, character_name: &str) -> String {
        match self {
            Tool::Crowbar => format!(
                "{} used their crowbar and forced open the door.",
                character_name
            ),
            Tool::Blowtorch => format!(
                "{} used their blowtorch and cut open the door.",
                character_name
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keycard;

#[derive(Serialize, Deserialize)]
pub struct Medkit;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CanWield;

#[derive(Debug, Serialize, Deserialize)]
pub struct Weapon(pub f32);

#[derive(Serialize, Deserialize)]
pub struct Price(pub i32);

// A type handy for spawning a variable type of item
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    FuelCan,
    FoodRation,
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

    pub fn noun_data(self) -> Noun {
        match self {
            Type::FuelCan => Noun::new("fuel can", "fuel cans"),
            Type::FoodRation => Noun::new("food ration", "food rations"),
            Type::Crowbar => Noun::new("crowbar", "crowbars"),
            Type::Blowtorch => Noun::new("blowtorch", "blowtorches"),
            Type::Keycard => Noun::new("keycard", "keycards"),
            Type::Knife => Noun::new("knife", "knives"),
            Type::Bat => Noun::new("bat", "bats"),
            Type::Sword => Noun::new("sword", "swords"),
            Type::Medkit => Noun::new("medkit", "medkits"),
            Type::MeteorChunk => Noun::new("meteor chunk", "meteor chunks"),
            Type::AncientCoin => Noun::new("ancient coin", "ancient coins"),
        }
    }

    pub fn symbol(self) -> Symbol {
        Symbol(match self {
            Type::FuelCan => 'f',
            Type::FoodRation => '%',
            Type::Crowbar => 'c',
            Type::Blowtorch => 'b',
            Type::Keycard => 'k',
            Type::Knife => 'K',
            Type::Bat => 'B',
            Type::Sword => 's',
            Type::Medkit => '+',
            Type::MeteorChunk => 'm',
            Type::AncientCoin => 'a',
        })
    }

    pub fn price(self) -> Option<i32> {
        match self {
            Type::FuelCan => Some(3500),
            Type::FoodRation => Some(500),
            Type::Crowbar => Some(2000),
            Type::Blowtorch => Some(7000),
            Type::Knife => Some(300),
            Type::Bat => Some(1000),
            Type::Sword => Some(5000),
            Type::Medkit => Some(4000),
            Type::MeteorChunk => Some(2500),
            Type::AncientCoin => Some(500),
            _ => None,
        }
    }
}

impl From<Type> for TextureType {
    fn from(item: Type) -> Self {
        TextureType::item(match item {
            Type::FuelCan => "fuel_can",
            Type::FoodRation => "food_ration",
            Type::Crowbar => "crowbar",
            Type::Blowtorch => "blowtorch",
            Type::Keycard => "keycard",
            Type::Knife => "knife",
            Type::Bat => "bat",
            Type::Sword => "sword",
            Type::Medkit => "medkit",
            Type::MeteorChunk => "meteor_chunk",
            Type::AncientCoin => "ancient_coin",
        })
    }
}

pub fn spawn(world: &mut World, item_type: Type, location: impl Component) -> Entity {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        location,
        Item,
        item_type.symbol(),
        TextureType::from(item_type),
        OrderWeight::Item,
        item_type.noun_data(),
    ));
    if let Some(price) = item_type.price() {
        builder.add(Price(price));
    }

    match item_type {
        Type::FuelCan => {
            builder.add(FuelCan);
        }
        Type::FoodRation => {
            builder.add(FoodRation);
        }
        Type::Crowbar => {
            builder.add_bundle((Tool::Crowbar, CanWield, Weapon(3.0)));
        }
        Type::Blowtorch => {
            builder.add(Tool::Blowtorch);
        }
        Type::Keycard => {
            builder.add(Keycard);
        }
        Type::Knife => {
            builder.add_bundle((CanWield, Weapon(3.0)));
        }
        Type::Bat => {
            builder.add_bundle((CanWield, Weapon(4.0)));
        }
        Type::Sword => {
            builder.add_bundle((CanWield, Weapon(5.0)));
        }
        Type::Medkit => {
            builder.add(Medkit);
        }
        _ => {}
    };
    world.spawn(builder.build())
}
