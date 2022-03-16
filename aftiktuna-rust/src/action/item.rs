use crate::area::Position;
use crate::view::DisplayInfo;
use crate::{action, Action};
use hecs::{Component, Entity, World};

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct InInventory;

pub fn has_item<C: Component>(world: &World) -> bool {
    world.query::<(&InInventory, &C)>().iter().len() > 0
}

pub fn take_all(world: &mut World, aftik: Entity) -> Result<String, String> {
    let aftik_pos = world.get::<Position>(aftik).unwrap().0;
    let (item, name) = world
        .query::<(&Position, &DisplayInfo, &Item)>()
        .iter()
        .filter(|(_, (pos, _, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, display_info, _))| (item, display_info.name().to_string()))
        .ok_or("There are no items to take here.")?;

    let result = take_item(item, &name, world, aftik)?;
    if world
        .query::<(&Position, &DisplayInfo, &Item)>()
        .iter()
        .any(|(_, (pos, _, _))| pos.is_in(aftik_pos.get_area()))
    {
        world.insert_one(aftik, Action::TakeAll).unwrap();
    }
    Ok(result)
}

pub fn take_item(
    item: Entity,
    item_name: &str,
    world: &mut World,
    aftik: Entity,
) -> Result<String, String> {
    let item_pos = world
        .get::<Position>(item)
        .map_err(|_| format!("You lost track of the {}.", item_name))?
        .0;

    action::try_move_aftik(world, aftik, item_pos)?;
    world
        .remove_one::<Position>(item)
        .expect("Tried removing position from item");
    world
        .insert_one(item, InInventory)
        .expect("Tried adding inventory data to item");

    Ok(format!("You picked up the {}.", item_name))
}
