use super::display::{ModelId, OrderWeight};
use crate::core::AttackSet;
use crate::core::name::{IndefiniteArticle, Noun};
use crate::view::text::Messages;
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StunAttack;

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
pub enum ItemType {
    FuelCan,
    FoodRation,
    Crowbar,
    Blowtorch,
    Knife,
    Bat,
    Sword,
    Medkit,
    MeteorChunk,
    AncientCoin,
    BlackOrb,
    FourLeafClover,
}

impl ItemType {
    pub fn variants() -> &'static [Self] {
        use ItemType::*;
        &[
            FuelCan,
            FoodRation,
            Crowbar,
            Blowtorch,
            Knife,
            Bat,
            Sword,
            Medkit,
            MeteorChunk,
            AncientCoin,
            BlackOrb,
            FourLeafClover,
        ]
    }
    pub fn spawn(self, world: &mut World, location: impl Component) -> Entity {
        spawn(world, self, self.price(), location)
    }

    pub fn noun_data(self) -> Noun {
        use ItemType::*;
        match self {
            FuelCan => Noun::new("fuel can", "fuel cans", IndefiniteArticle::A),
            FoodRation => Noun::new("food ration", "food rations", IndefiniteArticle::A),
            Crowbar => Noun::new("crowbar", "crowbars", IndefiniteArticle::A),
            Blowtorch => Noun::new("blowtorch", "blowtorches", IndefiniteArticle::A),
            Knife => Noun::new("knife", "knives", IndefiniteArticle::A),
            Bat => Noun::new("bat", "bats", IndefiniteArticle::A),
            Sword => Noun::new("sword", "swords", IndefiniteArticle::A),
            Medkit => Noun::new("medkit", "medkits", IndefiniteArticle::A),
            MeteorChunk => Noun::new("meteor chunk", "meteor chunks", IndefiniteArticle::A),
            AncientCoin => Noun::new("ancient coin", "ancient coins", IndefiniteArticle::An),
            BlackOrb => Noun::new("black orb", "black orbs", IndefiniteArticle::A),
            FourLeafClover => Noun::new(
                "four-leaf clover",
                "four-leaf clovers",
                IndefiniteArticle::A,
            ),
        }
    }

    pub fn price(self) -> Option<Price> {
        use ItemType::*;
        match self {
            FuelCan => Some(3500),
            FoodRation => Some(500),
            Crowbar => Some(2000),
            Blowtorch => Some(6000),
            Knife => Some(300),
            Bat => Some(1000),
            Sword => Some(5000),
            Medkit => Some(4000),
            MeteorChunk => Some(2500),
            AncientCoin => Some(500),
            BlackOrb => Some(8000),
            _ => None,
        }
        .map(Price)
    }
}

impl From<ItemType> for ModelId {
    fn from(item: ItemType) -> Self {
        use ItemType::*;
        ModelId::item(match item {
            FuelCan => "fuel_can",
            FoodRation => "food_ration",
            Crowbar => "crowbar",
            Blowtorch => "blowtorch",
            Knife => "knife",
            Bat => "bat",
            Sword => "sword",
            Medkit => "medkit",
            MeteorChunk => "meteor_chunk",
            AncientCoin => "ancient_coin",
            BlackOrb => "black_orb",
            FourLeafClover => "four_leaf_clover",
        })
    }
}

pub fn spawn(
    world: &mut World,
    item_type: ItemType,
    price: Option<Price>,
    location: impl Component,
) -> Entity {
    let mut builder = EntityBuilder::new();
    builder.add_bundle((
        location,
        Item,
        ModelId::from(item_type),
        OrderWeight::Item,
        item_type.noun_data(),
    ));
    if let Some(price) = price {
        builder.add(price);
    }

    match item_type {
        ItemType::FuelCan => {
            builder.add(FuelCan);
        }
        ItemType::FoodRation => {
            builder.add(FoodRation);
        }
        ItemType::Crowbar => {
            builder.add_bundle((Tool::Crowbar, CanWield, Weapon(3.0), AttackSet::Light));
        }
        ItemType::Blowtorch => {
            builder.add(Tool::Blowtorch);
        }
        ItemType::Knife => {
            builder.add_bundle((CanWield, Weapon(3.0), AttackSet::Quick));
        }
        ItemType::Bat => {
            builder.add_bundle((CanWield, StunAttack, Weapon(3.0), AttackSet::Intense));
        }
        ItemType::Sword => {
            builder.add_bundle((CanWield, Weapon(5.0), AttackSet::Quick));
        }
        ItemType::Medkit => {
            builder.add(Medkit);
        }
        ItemType::BlackOrb => {
            builder.add(Usable::BlackOrb);
        }
        ItemType::FourLeafClover => {
            builder.add(FourLeafClover);
        }
        _ => {}
    };
    world.spawn(builder.build())
}

pub fn description(item_ref: EntityRef) -> Vec<String> {
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
    messages.into_text()
}
