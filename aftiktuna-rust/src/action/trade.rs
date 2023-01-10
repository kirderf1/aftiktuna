use crate::action::item::InInventory;
use crate::action::CrewMember;
use crate::position::Pos;
use crate::view::DisplayInfo;
use crate::{item, position};
use hecs::{Entity, Ref, World};

pub struct Points(pub i32);

pub struct Shopkeeper(pub item::Type, pub i32);

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

pub fn buy(world: &mut World, performer: Entity) -> Result<String, String> {
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let shopkeeper = world
        .get::<&IsTrading>(performer)
        .map_err(|_| format!("{} is not currently trading.", performer_name))?
        .0;

    let (item_type, cost) = world
        .get::<&Shopkeeper>(shopkeeper)
        .map(|shopkeeper| (shopkeeper.0, shopkeeper.1))
        .unwrap();

    try_spend_points(world, crew, cost)?;

    let item = item::spawn(world, item_type, InInventory(performer));

    Ok(format!(
        "{} bought 1 {}.",
        performer_name,
        DisplayInfo::find_name(world, item)
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
