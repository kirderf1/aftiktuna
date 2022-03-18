use crate::position::Position;
use crate::view::DisplayInfo;
use crate::{position, Action};
use hecs::{Component, Entity, With, World};

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct InInventory;

pub fn has_item<C: Component>(world: &World) -> bool {
    world.query::<With<C, With<InInventory, ()>>>().iter().len() > 0
}

pub fn take_all(world: &mut World, aftik: Entity) -> Result<String, String> {
    let aftik_pos = world.get::<Position>(aftik).unwrap().0;
    let (item, name) = world
        .query::<With<Item, (&Position, &DisplayInfo)>>()
        .iter()
        .filter(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, display_info))| (item, display_info.name().to_string()))
        .ok_or("There are no items to take here.")?;

    let result = take_item(world, aftik, item, &name)?;
    if world
        .query::<With<Item, (&Position, &DisplayInfo)>>()
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
    let item_pos = world
        .get::<Position>(item)
        .map_err(|_| format!("You lost track of the {}.", item_name))?
        .0;

    position::try_move_aftik(world, aftik, item_pos)?;
    world
        .remove_one::<Position>(item)
        .expect("Tried removing position from item");
    world
        .insert_one(item, InInventory)
        .expect("Tried adding inventory data to item");

    Ok(format!("You picked up the {}.", item_name))
}
