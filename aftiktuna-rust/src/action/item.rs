use crate::action::{Action, Aftik};
use crate::position::{try_move, Pos};
use crate::view::DisplayInfo;
use hecs::{Component, Entity, Or, With, World};

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct InInventory;

pub fn is_holding<C: Component>(world: &World) -> bool {
    world
        .query::<Or<With<InInventory, ()>, With<Wielded, ()>>>()
        .with::<C>()
        .iter()
        .len()
        > 0
}

pub fn consume_one<C: Component>(world: &mut World) -> Option<()> {
    let (item, _) = world
        .query::<()>()
        .with::<InInventory>()
        .with::<C>()
        .iter()
        .next()?;
    world.despawn(item).ok()
}

pub fn take_all(world: &mut World, aftik: Entity) -> Result<String, String> {
    let aftik_pos = *world.get::<Pos>(aftik).unwrap();
    let (item, name) = world
        .query::<With<Item, (&Pos, &DisplayInfo)>>()
        .iter()
        .filter(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, display_info))| (item, display_info.definite_name().to_string()))
        .ok_or("There are no items to take here.")?;

    let result = take_item(world, aftik, item, &name)?;
    if world
        .query::<With<Item, (&Pos, &DisplayInfo)>>()
        .iter()
        .any(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
    {
        world.insert_one(aftik, Action::TakeAll).unwrap();
    }
    Ok(result)
}

pub fn take_item(
    world: &mut World,
    aftik: Entity,
    item: Entity,
    item_name: &str,
) -> Result<String, String> {
    let aftik_name = DisplayInfo::find_definite_name(world, aftik);
    let item_pos = *world
        .get::<Pos>(item)
        .map_err(|_| format!("{} lost track of {}.", aftik_name, item_name))?;

    try_move(world, aftik, item_pos)?;
    world
        .remove_one::<Pos>(item)
        .expect("Tried removing position from item");
    world
        .insert_one(item, InInventory)
        .expect("Tried adding inventory data to item");

    Ok(format!("{} picked up {}.", aftik_name, item_name))
}

#[derive(Debug, Default)]
pub struct CanWield;

#[derive(Debug)]
struct Wielded;

pub fn get_wielded(world: &World, entity: Entity) -> Option<Entity> {
    world.get::<Aftik>(entity).ok()?;
    world
        .query::<With<Wielded, ()>>()
        .iter()
        .next()
        .map(|(item, _)| item)
}

pub fn wield(
    world: &mut World,
    aftik: Entity,
    item: Entity,
    item_name: &str,
) -> Result<String, String> {
    let aftik_name = DisplayInfo::find_definite_name(world, aftik);

    if world.get::<InInventory>(item).is_ok() {
        unwield_if_needed(world, aftik);
        world.remove_one::<InInventory>(item).unwrap();
        world.insert_one(item, Wielded).unwrap();

        Ok(format!("{} wielded a {}.", aftik_name, item_name))
    } else {
        let item_pos = *world
            .get::<Pos>(item)
            .map_err(|_| format!("{} lost track of {}.", aftik_name, item_name))?;
        try_move(world, aftik, item_pos)?;

        unwield_if_needed(world, aftik);
        world
            .remove_one::<Pos>(item)
            .expect("Tried removing position from item");
        world
            .insert_one(item, Wielded)
            .expect("Tried adding inventory data to item");

        Ok(format!(
            "{} picked up and wielded the {}.",
            aftik_name, item_name
        ))
    }
}

fn unwield_if_needed(world: &mut World, entity: Entity) {
    if let Some(item) = get_wielded(world, entity) {
        world.remove_one::<Wielded>(item).unwrap();
        world.insert_one(item, InInventory).unwrap();
    }
}
