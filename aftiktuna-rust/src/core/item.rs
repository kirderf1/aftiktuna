use super::display::{ModelId, OrderWeight};
use crate::core::name::{IndefiniteArticle, Noun};
use crate::core::{AttackSet, WeaponProperties};
use crate::view::text::Messages;
use hecs::{Component, Entity, EntityBuilder, EntityRef, World};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq)]
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

    pub fn matches(self, item_type: ItemType) -> bool {
        item_type
            == match self {
                Tool::Crowbar => ItemType::Crowbar,
                Tool::Blowtorch => ItemType::Blowtorch,
            }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CanWield;

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

    pub fn weapon_properties(self) -> Option<WeaponProperties> {
        match self {
            Self::Crowbar => Some(WeaponProperties {
                damage_mod: 3.0,
                attack_set: AttackSet::Light,
                stun_attack: false,
            }),
            Self::Knife => Some(WeaponProperties {
                damage_mod: 3.0,
                attack_set: AttackSet::Quick,
                stun_attack: false,
            }),
            Self::Bat => Some(WeaponProperties {
                damage_mod: 3.0,
                attack_set: AttackSet::Intense,
                stun_attack: true,
            }),
            Self::Sword => Some(WeaponProperties {
                damage_mod: 5.0,
                attack_set: AttackSet::Quick,
                stun_attack: false,
            }),
            _ => None,
        }
    }

    pub fn is_usable(self) -> bool {
        matches!(self, Self::Medkit | Self::BlackOrb)
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
        item_type,
        ModelId::from(item_type),
        OrderWeight::Item,
        item_type.noun_data(),
    ));
    if let Some(price) = price {
        builder.add(price);
    }

    if item_type.weapon_properties().is_some() {
        builder.add(CanWield);
    }

    world.spawn(builder.build())
}

pub fn description(item_ref: EntityRef) -> Vec<String> {
    let mut messages = Messages::default();
    messages.add(format!("{}:", item_ref.get::<&Noun>().unwrap().singular()));

    let item_type = *item_ref.get::<&ItemType>().unwrap();

    if let Some(weapon_properties) = item_type.weapon_properties() {
        messages.add(format!("Weapon value: {}", weapon_properties.damage_mod));
    }
    match item_type {
        ItemType::FuelCan => messages.add("Used to refuel the ship."),
        ItemType::FoodRation => messages.add("May be eaten by crew members while travelling to their next location to recover health."),
        ItemType::Crowbar => messages.add("Used to force open doors that are stuck."),
        ItemType::Blowtorch => messages.add("Used to cut apart any door that won't open."),
        ItemType::Medkit => messages.add("Used to recover some health of the user."),
        ItemType::BlackOrb => messages.add("A mysterious object that when used, might change the user in some way."),
        ItemType::FourLeafClover => messages.add("A mysterious object said to bring luck to whoever finds it."),
        _ => {}
    }
    if item_ref.satisfies::<&Price>() {
        messages.add("Can be sold at a store.");
    }
    messages.into_text()
}
