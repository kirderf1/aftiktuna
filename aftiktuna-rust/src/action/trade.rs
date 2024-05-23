use crate::action;
use crate::action::CrewMember;
use crate::core::inventory::Held;
use crate::core::item::Price;
use crate::core::name::{self, NameData};
use crate::core::position::Pos;
use crate::core::{item, position, IsTrading, Points, Shopkeeper, StoreStock};
use hecs::{Entity, Ref, World};

pub fn get_shop_info(world: &World, character: Entity) -> Option<Ref<Shopkeeper>> {
    let shopkeeper = world.get::<&IsTrading>(character).ok()?.0;
    world.get::<&Shopkeeper>(shopkeeper).ok()
}

pub fn trade(world: &mut World, performer: Entity, shopkeeper: Entity) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();

    let shop_pos = *world
        .get::<&Pos>(shopkeeper)
        .map_err(|_| format!("{performer_name} lost track of the shopkeeper."))?;
    world.get::<&Shopkeeper>(shopkeeper).unwrap();

    position::move_adjacent(world, performer, shop_pos)?;

    world.insert_one(performer, IsTrading(shopkeeper)).unwrap();

    action::ok(format!(
        "{performer_name} starts trading with the shopkeeper. \"Welcome to the store. What do you want to buy?\"",
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

    let stock = world
        .get::<&Shopkeeper>(shopkeeper)
        .ok()
        .and_then(|shopkeeper| find_stock(&shopkeeper, item_type))
        .ok_or_else(|| "The item is not in stock.".to_string())?;

    if amount < 1 {
        return Err("Tried to purchase a non-positive number of items.".to_string());
    }

    try_spend_points(world, crew, stock.price.buy_price() * i32::from(amount))?;

    for _ in 0..amount {
        item::spawn(
            world,
            stock.item,
            Some(stock.price),
            Held::in_inventory(performer),
        );
    }

    action::ok(format!(
        "{} bought {}.",
        performer_name,
        stock.item.noun_data().with_count(amount),
    ))
}

fn find_stock(shopkeeper: &Shopkeeper, item_type: item::Type) -> Option<StoreStock> {
    shopkeeper
        .0
        .iter()
        .find(|priced| priced.item == item_type)
        .cloned()
}

pub fn sell(world: &mut World, performer: Entity, items: Vec<Entity>) -> action::Result {
    let mut value = 0;
    for item in &items {
        world
            .get::<&Held>(*item)
            .ok()
            .filter(|held| held.held_by(performer))
            .ok_or_else(|| "Item to sell is not being held!".to_string())?;
        value += world
            .get::<&Price>(*item)
            .map_err(|_| "That item can not be sold.".to_string())?
            .sell_price();
    }

    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let performer_name = NameData::find(world, performer).definite();
    let item_list = name::as_grouped_text_list(
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
