use crate::action;
use crate::action::Action;
use crate::core::inventory::Held;
use crate::core::item::{Item, Medkit};
use crate::core::position::{try_move, try_move_adjacent, Pos};
use crate::core::status::Health;
use crate::core::{inventory, status};
use crate::view::name::{NameData, NameQuery};
use crate::view::DisplayInfo;
use hecs::{Entity, World};

pub fn take_all(world: &mut World, aftik: Entity) -> action::Result {
    let aftik_pos = *world.get::<&Pos>(aftik).unwrap();
    let (item, name) = world
        .query::<(&Pos, NameQuery)>()
        .with::<&Item>()
        .iter()
        .filter(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, query))| (item, NameData::from(query)))
        .ok_or("There are no items to take here.")?;

    let result = take_item(world, aftik, item, name)?;
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
    performer: Entity,
    item: Entity,
    item_name: NameData,
) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();
    let item_pos = *world
        .get::<&Pos>(item)
        .map_err(|_| format!("{} lost track of {}.", performer_name, item_name.definite()))?;

    try_move(world, performer, item_pos)?;
    world
        .exchange_one::<Pos, _>(item, Held::in_inventory(performer))
        .expect("Tried moving item to inventory");

    action::ok(format!(
        "{} picked up {}.",
        performer_name,
        item_name.definite()
    ))
}

pub fn give_item(
    world: &mut World,
    performer: Entity,
    item: Entity,
    receiver: Entity,
) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();
    let receiver_name = NameData::find(world, receiver).definite();

    if world
        .get::<&Held>(item)
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

    try_move_adjacent(world, performer, receiver_pos)?;

    world
        .insert_one(item, Held::in_inventory(receiver))
        .unwrap();

    action::ok(format!(
        "{} gave {} a {}.",
        performer_name,
        receiver_name,
        NameData::find(world, item).base()
    ))
}

pub fn wield(
    world: &mut World,
    performer: Entity,
    item: Entity,
    item_name: NameData,
) -> action::Result {
    let performer_name = NameData::find(world, performer).definite();

    if inventory::is_in_inventory(world, item, performer) {
        inventory::unwield_if_needed(world, performer);
        world.insert_one(item, Held::in_hand(performer)).unwrap();

        action::ok(format!(
            "{} wielded {}.",
            performer_name,
            item_name.definite()
        ))
    } else {
        let item_pos = *world
            .get::<&Pos>(item)
            .map_err(|_| format!("{} lost track of {}.", performer_name, item_name.definite()))?;
        try_move(world, performer, item_pos)?;

        inventory::unwield_if_needed(world, performer);
        world
            .exchange_one::<Pos, _>(item, Held::in_hand(performer))
            .expect("Tried moving item");

        action::ok(format!(
            "{} picked up and wielded {}.",
            performer_name,
            item_name.definite()
        ))
    }
}

pub fn use_medkit(world: &mut World, performer: Entity, item: Entity) -> action::Result {
    world
        .get::<&Medkit>(item)
        .map_err(|_| "The medkit is missing.".to_string())?;
    world
        .get::<&Held>(item)
        .ok()
        .filter(|held| held.held_by(performer))
        .ok_or_else(|| "The medkit is missing.".to_string())?;

    if !world.get::<&Health>(performer).unwrap().is_hurt() {
        return Err(format!(
            "{} no longer needs to use a medkit.",
            NameData::find(world, performer).definite()
        ));
    }

    world
        .get::<&mut Health>(performer)
        .unwrap()
        .restore_fraction(0.5);
    world.despawn(item).unwrap();

    action::ok(format!(
        "{} used a medkit and recovered some health.",
        NameData::find(world, performer).definite()
    ))
}
