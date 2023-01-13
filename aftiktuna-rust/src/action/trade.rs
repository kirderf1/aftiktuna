use crate::action::item::InInventory;
use crate::action::CrewMember;
use crate::item::Price;
use crate::position::Pos;
use crate::view::DisplayInfo;
use crate::{item, position};
use hecs::{Entity, Ref, World};
use std::ops::Deref;

pub struct Points(pub i32);

pub struct Shopkeeper(pub Vec<PricedItem>);

#[derive(Clone)]
pub struct PricedItem {
    pub item: item::Type,
    pub price: i32,
}

struct IsTrading(Entity);

pub fn get_shop_info(world: &World, character: Entity) -> Option<Ref<Shopkeeper>> {
    let shopkeeper = world.get::<&IsTrading>(character).ok()?.0;
    world.get::<&Shopkeeper>(shopkeeper).ok()
}

pub fn trade(world: &mut World, performer: Entity, shopkeeper: Entity) -> Result<String, String> {
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    let performer_pos = *world.get::<&Pos>(performer).unwrap();

    let shop_pos = *world
        .get::<&Pos>(shopkeeper)
        .map_err(|_| format!("{} lost track of the shopkeeper.", performer_name))?;
    world.get::<&Shopkeeper>(shopkeeper).unwrap();

    position::try_move(
        world,
        performer,
        shop_pos.get_adjacent_towards(performer_pos),
    )?;

    world.insert_one(performer, IsTrading(shopkeeper)).unwrap();

    Ok(format!(
        "{} starts trading with the shopkeeper.",
        performer_name,
    ))
}

pub fn buy(world: &mut World, performer: Entity, item_type: item::Type) -> Result<String, String> {
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let shopkeeper = world
        .get::<&IsTrading>(performer)
        .map_err(|_| format!("{} is not currently trading.", performer_name))?
        .0;

    let priced_item = world
        .get::<&Shopkeeper>(shopkeeper)
        .ok()
        .and_then(|shopkeeper| find_priced_item(shopkeeper.deref(), item_type))
        .ok_or_else(|| "The item is not in stock.".to_string())?;

    try_spend_points(world, crew, priced_item.price)?;

    let item = item::spawn(world, priced_item.item, InInventory(performer));

    Ok(format!(
        "{} bought 1 {}.",
        performer_name,
        DisplayInfo::find_name(world, item)
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

pub fn sell(world: &mut World, performer: Entity, item: Entity) -> Result<String, String> {
    world
        .get::<&InInventory>(item)
        .ok()
        .filter(|in_inventory| in_inventory.held_by(performer))
        .ok_or_else(|| "Item to sell is not being held!".to_string())?;
    let price = world
        .get::<&Price>(item)
        .map_err(|_| "That item can not be sold.".to_string())?
        .0;
    let price = price - price / 4;
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    let item_name = DisplayInfo::find_name(world, item);

    world.get::<&mut Points>(crew).unwrap().0 += price;
    world.despawn(item).unwrap();
    Ok(format!(
        "{} sold a {} for {}.",
        performer_name, item_name, price
    ))
}

pub fn exit(world: &mut World, performer: Entity) -> Result<String, String> {
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    world
        .remove_one::<IsTrading>(performer)
        .map_err(|_| format!("{} is already not trading.", performer_name,))?;

    Ok(format!(
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
