use crate::action::combat::IsFoe;
use crate::action::trade::{PricedItem, Shopkeeper};
use crate::action::CrewMember;
use crate::item;
use crate::position::{MovementBlocking, Pos};
use crate::status::{Health, Stamina, Stats};
use crate::view::DisplayInfo;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

pub fn spawn_aftik(world: &mut World, crew: Entity, name: &str, stats: Stats) -> Entity {
    world.spawn((
        DisplayInfo::from_name(name.chars().next().unwrap(), name, 10),
        CrewMember(crew),
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ))
}

pub fn place_goblin(world: &mut World, pos: Pos) {
    let stats = Stats::new(2, 4, 10);
    world.spawn((
        DisplayInfo::from_noun('G', "goblin", 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
}

pub fn place_eyesaur(world: &mut World, pos: Pos) {
    let stats = Stats::new(7, 7, 4);
    world.spawn((
        DisplayInfo::from_noun('E', "eyesaur", 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
}

pub fn place_azureclops(world: &mut World, pos: Pos) {
    let stats = Stats::new(15, 10, 4);
    world.spawn((
        DisplayInfo::from_noun('Z', "azureclops", 10),
        pos,
        MovementBlocking,
        IsFoe,
        Health::with_max(&stats),
        Stamina::with_max(&stats),
        stats,
    ));
}

pub fn place_shopkeeper(world: &mut World, pos: Pos, shop_items: &[ShopItem]) {
    world.spawn((
        DisplayInfo::from_noun('S', "shopkeeper", 15),
        pos,
        Shopkeeper(shop_items.iter().map(to_priced_item).collect::<Vec<_>>()),
    ));
}

fn to_priced_item(shop_item: &ShopItem) -> PricedItem {
    match shop_item {
        ShopItem::FuelCan => PricedItem {
            item: item::Type::FuelCan,
            price: 3500,
        },
        ShopItem::Knife => PricedItem {
            item: item::Type::Knife,
            price: 300,
        },
        ShopItem::Bat => PricedItem {
            item: item::Type::Bat,
            price: 1000,
        },
        ShopItem::Sword => PricedItem {
            item: item::Type::Sword,
            price: 3000,
        },
        ShopItem::MeteorChunk => PricedItem {
            item: item::Type::MeteorChunk,
            price: 2500,
        },
        ShopItem::AncientCoin => PricedItem {
            item: item::Type::AncientCoin,
            price: 500,
        },
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopItem {
    FuelCan,
    Knife,
    Bat,
    Sword,
    MeteorChunk,
    AncientCoin,
}
