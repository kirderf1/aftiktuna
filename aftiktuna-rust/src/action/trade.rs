use crate::action::item::Held;
use crate::action::CrewMember;
use crate::item::Price;
use crate::position::Pos;
use crate::view::{as_grouped_text_list, NameData};
use crate::{action, item, position};
use hecs::{Entity, Ref, World};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Points(pub i32);

#[derive(Serialize, Deserialize)]
pub struct Shopkeeper(pub Vec<PricedItem>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricedItem {
    pub item: item::Type,
    pub price: i32,
}

#[derive(Serialize, Deserialize)]
pub struct IsTrading(Entity);

pub fn get_shop_info(world: &World, character: Entity) -> Option<Ref<Shopkeeper>> {
    let shopkeeper = world.get::<&IsTrading>(character).ok()?.0;
    world.get::<&Shopkeeper>(shopkeeper).ok()
}

pub fn trade(world: &mut World, performer: Entity, shopkeeper: Entity) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();

    let shop_pos = *world
        .get::<&Pos>(shopkeeper)
        .map_err(|_| format!("{} lost track of the shopkeeper.", performer_name))?;
    world.get::<&Shopkeeper>(shopkeeper).unwrap();

    position::try_move_adjacent(world, performer, shop_pos)?;

    world.insert_one(performer, IsTrading(shopkeeper)).unwrap();

    action::ok(format!(
        "{} starts trading with the shopkeeper.",
        performer_name,
    ))
}

pub fn buy(
    world: &mut World,
    performer: Entity,
    item_type: item::Type,
    amount: u16,
) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let shopkeeper = world
        .get::<&IsTrading>(performer)
        .map_err(|_| format!("{} is not currently trading.", performer_name))?
        .0;

    let priced_item = world
        .get::<&Shopkeeper>(shopkeeper)
        .ok()
        .and_then(|shopkeeper| find_priced_item(&*shopkeeper, item_type))
        .ok_or_else(|| "The item is not in stock.".to_string())?;

    if amount < 1 {
        return Err("Tried to purchase a non-positive number of items.".to_string());
    }

    try_spend_points(world, crew, priced_item.price * i32::from(amount))?;

    for _ in 0..amount {
        item::spawn(world, priced_item.item, Held::in_inventory(performer));
    }

    action::ok(format!(
        "{} bought {}.",
        performer_name,
        priced_item.item.noun_data().with_count(amount),
    ))
}

fn find_priced_item(shopkeeper: &Shopkeeper, item_type: item::Type) -> Option<PricedItem> {
    shopkeeper
        .0
        .iter()
        .filter(|priced| priced.item == item_type)
        .map(PricedItem::clone)
        .next()
}

pub fn sell(world: &mut World, performer: Entity, items: Vec<Entity>) -> action::Result {
    let mut price = 0;
    for item in &items {
        world
            .get::<&Held>(*item)
            .ok()
            .filter(|held| held.held_by(performer))
            .ok_or_else(|| "Item to sell is not being held!".to_string())?;
        price += world
            .get::<&Price>(*item)
            .map_err(|_| "That item can not be sold.".to_string())?
            .0;
    }
    // Their sell value is a portion of the buy price
    let value = price - price / 4;

    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let performer_name = NameData::find(world, performer).definite();
    let item_list = as_grouped_text_list(
        items
            .iter()
            .map(|item| NameData::find(world, *item))
            .collect(),
    );

    world.get::<&mut Points>(crew).unwrap().0 += value;
    for item in items {
        world.despawn(item).unwrap();
    }

    action::ok(format!(
        "{} sold {} for {}.",
        performer_name, item_list, value
    ))
}

pub fn exit(world: &mut World, performer: Entity) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();
    world
        .remove_one::<IsTrading>(performer)
        .map_err(|_| format!("{} is already not trading.", performer_name,))?;

    action::ok(format!(
        "{} stops trading with the shopkeeper.",
        performer_name
    ))
}

fn try_spend_points(world: &mut World, crew: Entity, points: i32) -> Result<(), String> {
    if let Ok(mut crew_points) = world.get::<&mut Points>(crew) {
        if crew_points.0 >= points {
            crew_points.0 -= points;
            Ok(())
        } else {
            Err("The crew cannot afford that.".to_string())
        }
    } else {
        Err("The crew is missing its wallet.".to_string())
    }
}
