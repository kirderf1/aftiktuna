use crate::action::{self, Error};
use crate::core::CrewMember;
use crate::core::area::{FuelAmount, ShipState, ShipStatus};
use crate::core::inventory::Held;
use crate::core::item::{self, ItemType, Price};
use crate::core::name::{self, NameData, NameIdData};
use crate::core::position::{self, Placement, PlacementQuery};
use crate::core::store::{IsTrading, Points, Shopkeeper, StoreStock};
use crate::view::text;
use hecs::{Entity, EntityRef, World};

pub fn trade(
    context: &mut action::Context,
    performer: Entity,
    shopkeeper: Entity,
) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let world = &mut context.state.world;
    let performer_name = NameData::find(world, performer, assets).definite();

    let shop_placement = world
        .query_one_mut::<PlacementQuery>(shopkeeper)
        .map(Placement::from)
        .map_err(|_| format!("{performer_name} lost track of the shopkeeper."))?;
    world.get::<&Shopkeeper>(shopkeeper).unwrap();

    position::move_adjacent_placement(world, performer, shop_placement, assets)?;

    world.insert_one(performer, IsTrading(shopkeeper)).unwrap();

    context.view_context.add_message_at(shop_placement.area(), format!(
        "{performer_name} starts trading with the shopkeeper. \"Welcome to the store. What do you want to buy?\"",
    ), context.state);
    Ok(action::Success)
}

pub fn buy(
    context: &mut action::Context,
    performer: Entity,
    item_type: ItemType,
    amount: u16,
) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let world = &mut context.state.world;
    let (item, price) = try_buy(world, performer, item_type, amount)?;

    for _ in 0..amount {
        item::spawn(world, item, Some(price), Held::in_inventory(performer));
    }

    context.view_context.view_buffer.add_change_message(
        format!(
            "{the_performer} bought {an_item}.",
            the_performer = NameData::find(world, performer, assets).definite(),
            an_item = assets
                .noun_data_map
                .lookup(&item.noun_id())
                .with_text_count(amount, name::ArticleKind::A),
        ),
        context.state,
    );
    Ok(action::Success)
}

fn try_buy(
    world: &mut World,
    performer: Entity,
    item_type: ItemType,
    amount: u16,
) -> Result<(ItemType, Price), String> {
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

fn find_stock(shopkeeper: &mut Shopkeeper, item_type: ItemType) -> Option<&mut StoreStock> {
    shopkeeper
        .0
        .iter_mut()
        .find(|priced| priced.item == item_type)
}

pub fn sell(
    context: &mut action::Context,
    performer: Entity,
    items: Vec<Entity>,
) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let world = &mut context.state.world;
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
        is_selling_fuel |= item_ref
            .get::<&ItemType>()
            .is_some_and(|item_type| *item_type == ItemType::FuelCan);
    }

    let performer_name = NameData::find(world, performer, assets).definite();

    if is_selling_fuel && !check_has_fuel_reserve(world, &items) {
        return Err(Error::private(format!(
            "{performer_name} does not want to sell their fuel can, since they need it to refuel their ship."
        )));
    }

    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let item_list = name::names_with_counts(
        items.iter().map(|item| NameIdData::find(world, *item)),
        name::ArticleKind::A,
        name::CountFormat::Text,
        assets,
    );

    world.get::<&mut Points>(crew).unwrap().0 += value;
    for item in items {
        world.despawn(item).unwrap();
    }

    context.view_context.view_buffer.add_change_message(
        format!(
            "{performer_name} sold {items} for {value}.",
            items = text::join_elements(item_list)
        ),
        context.state,
    );
    Ok(action::Success)
}

fn check_has_fuel_reserve(world: &World, excluding_items: &[Entity]) -> bool {
    let mut query = world.query::<&ShipState>();
    let Some((_, ship_state)) = query.iter().next() else {
        return true;
    };
    let ShipStatus::NeedFuel(fuel_amount) = ship_state.status else {
        return true;
    };

    let amount_needed = match fuel_amount {
        FuelAmount::OneCan => 1,
        FuelAmount::TwoCans => 2,
    };
    world
        .query::<(&ItemType, &Held)>()
        .iter()
        .filter(|&(item, (item_type, held))| {
            *item_type == ItemType::FuelCan
                && !excluding_items.contains(&item)
                && world.satisfies::<&CrewMember>(held.holder).unwrap_or(false)
        })
        .count()
        >= amount_needed
}

pub fn exit(context: &mut action::Context, performer: Entity) -> action::Result {
    let performer_name = NameData::find(
        &context.state.world,
        performer,
        context.view_context.view_buffer.assets,
    )
    .definite();
    context
        .state
        .world
        .remove_one::<IsTrading>(performer)
        .map_err(|_| format!("{performer_name} is already not trading."))?;

    context.view_context.view_buffer.add_change_message(
        format!("{performer_name} stops trading with the shopkeeper."),
        context.state,
    );
    Ok(action::Success)
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
