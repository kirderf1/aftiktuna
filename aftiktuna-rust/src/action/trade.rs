use crate::action::{self, Error};
use crate::core::area::{FuelAmount, Ship, ShipStatus};
use crate::core::inventory::Held;
use crate::core::item::{FuelCan, Price};
use crate::core::name::{self, NameData};
use crate::core::position::{self, Pos};
use crate::core::store::{IsTrading, Points, Shopkeeper, StoreStock};
use crate::core::{item, CrewMember};
use crate::view::text;
use hecs::{Entity, EntityRef, Ref, World};

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
    let (item, price) = try_buy(world, performer, item_type, amount)?;

    for _ in 0..amount {
        item::spawn(world, item, Some(price), Held::in_inventory(performer));
    }

    action::ok(format!(
        "{the_performer} bought {an_item}.",
        the_performer = NameData::find(world, performer).definite(),
        an_item = item.noun_data().with_text_count(amount, name::Article::A),
    ))
}

fn try_buy(
    world: &mut World,
    performer: Entity,
    item_type: item::Type,
    amount: u16,
) -> Result<(item::Type, Price), String> {
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let shopkeeper = world
        .get::<&IsTrading>(performer)
        .map_err(|_| "Tried to buy while not trading.")?
        .0;
    let mut shopkeeper = world.get::<&mut Shopkeeper>(shopkeeper).unwrap();
    let stock = find_stock(&mut shopkeeper, item_type).ok_or("The item is not in stock.")?;
    if amount < 1 {
        return Err("Tried to purchase a non-positive number of items.".to_owned());
    }

    let new_quantity = stock
        .quantity
        .subtracted(amount)
        .ok_or("Tried buying more than what was in stock.")?;
    try_spend_points(
        world.entity(crew).unwrap(),
        stock.price.buy_price() * i32::from(amount),
    )?;
    stock.quantity = new_quantity;

    Ok((stock.item, stock.price))
}

fn find_stock(shopkeeper: &mut Shopkeeper, item_type: item::Type) -> Option<&mut StoreStock> {
    shopkeeper
        .0
        .iter_mut()
        .find(|priced| priced.item == item_type)
}

pub fn sell(world: &mut World, performer: Entity, items: Vec<Entity>) -> action::Result {
    let mut value = 0;
    let mut is_selling_fuel = false;
    for &item in &items {
        let item_ref = world
            .entity(item)
            .map_err(|_| "One of the items being sold no longer exists.")?;
        item_ref
            .get::<&Held>()
            .filter(|held| held.held_by(performer))
            .ok_or("Item to sell is not being held!")?;
        value += item_ref
            .get::<&Price>()
            .ok_or("That item can not be sold.")?
            .sell_price();
        is_selling_fuel |= item_ref.satisfies::<&FuelCan>();
    }

    let performer_name = NameData::find(world, performer).definite();

    if is_selling_fuel && !check_has_fuel_reserve(world, &items) {
        return Err(Error::private(format!("{performer_name} does not want to sell their fuel can, since they need it to refuel their ship.")));
    }

    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let item_list = name::names_with_counts(
        items.iter().map(|item| NameData::find(world, *item)),
        name::Article::A,
        name::CountFormat::Text,
    );

    world.get::<&mut Points>(crew).unwrap().0 += value;
    for item in items {
        world.despawn(item).unwrap();
    }

    action::ok(format!(
        "{performer_name} sold {items} for {value}.",
        items = text::join_elements(item_list)
    ))
}

fn check_has_fuel_reserve(world: &World, excluding_items: &[Entity]) -> bool {
    let mut query = world.query::<&Ship>();
    let Some((_, ship)) = query.iter().next() else {
        return true;
    };
    let ShipStatus::NeedFuel(fuel_amount) = ship.status else {
        return true;
    };

    let amount_needed = match fuel_amount {
        FuelAmount::OneCan => 1,
        FuelAmount::TwoCans => 2,
    };
    world
        .query::<&Held>()
        .with::<&FuelCan>()
        .iter()
        .filter(|(item, held)| {
            !excluding_items.contains(item)
                && world.satisfies::<&CrewMember>(held.holder).unwrap_or(false)
        })
        .count()
        >= amount_needed
}

pub fn exit(world: &mut World, performer: Entity) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();
    world
        .remove_one::<IsTrading>(performer)
        .map_err(|_| format!("{} is already not trading.", performer_name,))?;

    action::ok(format!(
        "{performer_name} stops trading with the shopkeeper.",
    ))
}

fn try_spend_points(crew_ref: EntityRef, points: i32) -> Result<(), String> {
    let mut crew_points = crew_ref
        .get::<&mut Points>()
        .ok_or("The crew is missing its wallet.")?;
    if crew_points.0 < points {
        return Err("The crew cannot afford that.".to_string());
    }

    crew_points.0 -= points;
    Ok(())
}
