use crate::action::{Action, CrewMember};
use crate::position::{try_move, Pos};
use crate::status;
use crate::view::DisplayInfo;
use hecs::{Component, Entity, World};

#[derive(Debug, Default)]
pub struct Item;

#[derive(Debug, Default)]
pub struct FuelCan;

#[derive(Debug)]
pub struct InInventory(Entity);

impl InInventory {
    pub fn held_by(&self, holder: Entity) -> bool {
        self.0 == holder
    }
}

pub fn is_holding<C: Component>(world: &World, entity: Entity) -> bool {
    world
        .query::<&Wielded>()
        .with::<&C>()
        .iter()
        .any(|(_, wielded)| wielded.0 == entity)
        || world
            .query::<&InInventory>()
            .with::<&C>()
            .iter()
            .any(|(_, in_inventory)| in_inventory.held_by(entity))
}

pub fn is_in_inventory(world: &World, item: Entity, holder: Entity) -> bool {
    world
        .get::<&InInventory>(item)
        .ok()
        .map_or(false, |in_inventory| in_inventory.held_by(holder))
}

pub fn get_inventory(world: &World, holder: Entity) -> Vec<Entity> {
    world
        .query::<&InInventory>()
        .iter()
        .filter(|(_, in_inventory)| in_inventory.held_by(holder))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>()
}

pub fn consume_one<C: Component>(world: &mut World, entity: Entity) -> Option<()> {
    let (item, _) = world
        .query::<&InInventory>()
        .with::<&C>()
        .iter()
        .find(|(_, in_inventory)| in_inventory.held_by(entity))?;
    world.despawn(item).ok()
}

pub fn take_all(world: &mut World, aftik: Entity) -> Result<String, String> {
    let aftik_pos = *world.get::<&Pos>(aftik).unwrap();
    let (item, name) = world
        .query::<(&Pos, &DisplayInfo)>()
        .with::<&Item>()
        .iter()
        .filter(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, display_info))| (item, display_info.definite_name().to_string()))
        .ok_or("There are no items to take here.")?;

    let result = take_item(world, aftik, item, &name)?;
    if world
        .query::<(&Pos, &DisplayInfo)>()
        .with::<&Item>()
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
        .get::<&Pos>(item)
        .map_err(|_| format!("{} lost track of {}.", aftik_name, item_name))?;

    try_move(world, aftik, item_pos)?;
    world
        .exchange_one::<Pos, _>(item, InInventory(aftik))
        .expect("Tried moving item to inventory");

    Ok(format!("{} picked up {}.", aftik_name, item_name))
}

pub fn give_item(
    world: &mut World,
    performer: Entity,
    item: Entity,
    receiver: Entity,
) -> Result<String, String> {
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    let receiver_name = DisplayInfo::find_definite_name(world, receiver);

    if world
        .get::<&InInventory>(item)
        .ok()
        .filter(|in_inv| in_inv.held_by(performer))
        .is_none()
    {
        return Err(format!(
            "{} lost track of the item they were going to give.",
            performer_name
        ));
    }

    let performer_pos = *world
        .get::<&Pos>(performer)
        .expect("Expected performer to have a position");
    let receiver_pos = *world.get::<&Pos>(receiver).map_err(|_| {
        format!(
            "{} disappeared before {} could interact with them.",
            receiver_name, performer_name
        )
    })?;

    if !performer_pos.is_in(receiver_pos.get_area()) {
        return Err(format!(
            "{} left before {} could interact with them.",
            receiver_name, performer_name
        ));
    }

    if !status::is_alive(receiver, world) {
        return Err(format!(
            "{} died before they could be given an item.",
            receiver_name
        ));
    }

    try_move(
        world,
        performer,
        receiver_pos.get_adjacent_towards(performer_pos),
    )?;

    world.insert_one(item, InInventory(receiver)).unwrap();

    Ok(format!(
        "{} gave {} a {}.",
        performer_name,
        receiver_name,
        DisplayInfo::find_name(world, item)
    ))
}

#[derive(Debug, Default)]
pub struct CanWield;

#[derive(Debug)]
struct Wielded(Entity);

pub fn get_wielded(world: &World, entity: Entity) -> Option<Entity> {
    world.get::<&CrewMember>(entity).ok()?;
    world
        .query::<&Wielded>()
        .iter()
        .find(|(_, wielded)| wielded.0 == entity)
        .map(|(item, _)| item)
}

pub fn wield(
    world: &mut World,
    aftik: Entity,
    item: Entity,
    item_name: &str,
) -> Result<String, String> {
    let aftik_name = DisplayInfo::find_definite_name(world, aftik);

    if is_in_inventory(world, item, aftik) {
        unwield_if_needed(world, aftik);
        world.remove_one::<InInventory>(item).unwrap();
        world.insert_one(item, Wielded(aftik)).unwrap();

        Ok(format!("{} wielded a {}.", aftik_name, item_name))
    } else {
        let item_pos = *world
            .get::<&Pos>(item)
            .map_err(|_| format!("{} lost track of {}.", aftik_name, item_name))?;
        try_move(world, aftik, item_pos)?;

        unwield_if_needed(world, aftik);
        world
            .exchange_one::<Pos, _>(item, Wielded(aftik))
            .expect("Tried moving item");

        Ok(format!(
            "{} picked up and wielded the {}.",
            aftik_name, item_name
        ))
    }
}

fn unwield_if_needed(world: &mut World, holder: Entity) {
    if let Some(item) = get_wielded(world, holder) {
        world
            .exchange_one::<Wielded, _>(item, InInventory(holder))
            .unwrap();
    }
}

pub fn drop_all_items(world: &mut World, entity: Entity) {
    let pos = *world.get::<&Pos>(entity).unwrap();
    let items = get_inventory(world, entity);
    for item in items {
        world.exchange_one::<InInventory, _>(item, pos).unwrap();
    }
    let wielded = get_wielded(world, entity);
    if let Some(item) = wielded {
        world.exchange_one::<Wielded, _>(item, pos).unwrap();
    }
}
