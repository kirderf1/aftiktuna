use super::display::{ModelId, OrderWeight, Symbol};
use super::name::Noun;
use crate::view::Messages;
use hecs::{Component, Entity, EntityBuilder, EntityRef, World};
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Usable {
    BlackOrb,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FourLeafClover;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CanWield;

#[derive(Debug, Serialize, Deserialize)]
pub struct Weapon(pub f32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Price(i32);

impl Price {
    pub fn buy_price(&self) -> i32 {
        self.0
    }

    pub fn sell_price(&self) -> i32 {
        self.0 - self.0 / 4
    }
}

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
    BlackOrb,
    FourLeafClover,
}

impl Type {
    pub fn spawn(self, world: &mut World, location: impl Component) -> Entity {
        spawn(world, self, self.price(), location)
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
            Type::BlackOrb => Noun::new("black orb", "black orbs"),
            Type::FourLeafClover => Noun::new("four-leaf clover", "four-leaf clovers"),
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
            Type::BlackOrb => 'o',
            Type::FourLeafClover => '*',
        })
    }

    pub fn price(self) -> Option<Price> {
        match self {
            Type::FuelCan => Some(3500),
            Type::FoodRation => Some(500),
            Type::Crowbar => Some(2000),
            Type::Blowtorch => Some(6000),
            Type::Knife => Some(300),
            Type::Bat => Some(1000),
            Type::Sword => Some(5000),
            Type::Medkit => Some(4000),
            Type::MeteorChunk => Some(2500),
            Type::AncientCoin => Some(500),
            Type::BlackOrb => Some(8000),
            _ => None,
        }
        .map(Price)
    }
}

impl From<Type> for ModelId {
    fn from(item: Type) -> Self {
        ModelId::item(match item {
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
            Type::BlackOrb => "black_orb",
            Type::FourLeafClover => "four_leaf_clover",
        })
    }
}

pub fn spawn(
    world: &mut World,
    item_type: Type,
    price: Option<Price>,
    location: impl Component,
) -> Entity {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        location,
        Item,
        item_type.symbol(),
        ModelId::from(item_type),
        OrderWeight::Item,
        item_type.noun_data(),
    ));
    if let Some(price) = price {
        builder.add(price);
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
        Type::BlackOrb => {
            builder.add(Usable::BlackOrb);
        }
        Type::FourLeafClover => {
            builder.add(FourLeafClover);
        }
        _ => {}
    };
    world.spawn(builder.build())
}

pub fn description(item_ref: EntityRef) -> Messages {
    let mut messages = Messages::default();
    messages.add(format!("{}:", item_ref.get::<&Noun>().unwrap().singular()));

    if let Some(weapon) = item_ref.get::<&Weapon>() {
        messages.add(format!("Weapon value: {}", weapon.0));
    }
    if item_ref.satisfies::<&FuelCan>() {
        messages.add("Used to refuel the ship.");
    }
    if item_ref.satisfies::<&FoodRation>() {
        messages.add("May be eaten by crew members while travelling to their next location to recover health.");
    }
    if let Some(tool) = item_ref.get::<&Tool>() {
        messages.add(match *tool {
            Tool::Crowbar => "Used to force open doors that are stuck.",
            Tool::Blowtorch => "Used to cut apart any door that won't open.",
        });
    }
    if item_ref.satisfies::<&Keycard>() {
        messages.add("Used to let the holder pass through a locked door.");
    }
    if item_ref.satisfies::<&Medkit>() {
        messages.add("Used to recover some health of the user.");
    }
    if let Some(usage) = item_ref.get::<&Usable>() {
        messages.add(match *usage {
            Usable::BlackOrb => {
                "A mysterious object that when used, might change the user in some way."
            }
        });
    }
    if item_ref.satisfies::<&FourLeafClover>() {
        messages.add("A mysterious object said to bring luck to whoever finds it.");
    }
    if item_ref.satisfies::<&Price>() {
        messages.add("Can be sold at a store.");
    }
    messages
}
