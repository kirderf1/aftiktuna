use crate::action::Action;
use crate::item::{Item, Medkit};
use crate::position::{try_move, Pos};
use crate::status::Health;
use crate::view::{DisplayInfo, NameData};
use crate::{action, status};
use hecs::{Component, Entity, World};

#[derive(Debug)]
pub struct Held {
    holder: Entity,
    in_hand: bool,
}

impl Held {
    pub fn held_by(&self, holder: Entity) -> bool {
        self.holder == holder
    }

    pub fn is_in_hand(&self) -> bool {
        self.in_hand
    }

    pub fn is_in_inventory(&self, holder: Entity) -> bool {
        self.held_by(holder) && !self.in_hand
    }

    pub fn in_inventory(holder: Entity) -> Self {
        Held {
            holder,
            in_hand: false,
        }
    }
}

pub fn is_holding<C: Component>(world: &World, holder: Entity) -> bool {
    world
        .query::<&Held>()
        .with::<&C>()
        .iter()
        .any(|(_, held)| held.held_by(holder))
}

pub fn is_in_inventory(world: &World, item: Entity, holder: Entity) -> bool {
    world
        .get::<&Held>(item)
        .ok()
        .map_or(false, |held| held.is_in_inventory(holder))
}

pub fn get_held(world: &World, holder: Entity) -> Vec<Entity> {
    world
        .query::<&Held>()
        .iter()
        .filter(|(_, held)| held.held_by(holder))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>()
}

pub fn get_inventory(world: &World, holder: Entity) -> Vec<Entity> {
    world
        .query::<&Held>()
        .iter()
        .filter(|(_, held)| held.is_in_inventory(holder))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>()
}

pub fn get_wielded(world: &World, holder: Entity) -> Option<Entity> {
    world
        .query::<&Held>()
        .iter()
        .find(|(_, held)| held.held_by(holder) && held.in_hand)
        .map(|(item, _)| item)
}

pub fn consume_one<C: Component>(world: &mut World, holder: Entity) -> Option<()> {
    let (item, _) = world
        .query::<&Held>()
        .with::<&C>()
        .iter()
        .find(|(_, held)| held.held_by(holder))?;
    world.despawn(item).ok()
}

pub fn take_all(world: &mut World, aftik: Entity) -> action::Result {
    let aftik_pos = *world.get::<&Pos>(aftik).unwrap();
    let (item, name) = world
        .query::<(&Pos, &NameData)>()
        .with::<&Item>()
        .iter()
        .filter(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, name_data))| (item, name_data.clone()))
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

    try_move(
        world,
        performer,
        receiver_pos.get_adjacent_towards(performer_pos),
    )?;

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

    if is_in_inventory(world, item, performer) {
        unwield_if_needed(world, performer);
        world.get::<&mut Held>(item).unwrap().in_hand = true;

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

        unwield_if_needed(world, performer);
        world
            .exchange_one::<Pos, _>(
                item,
                Held {
                    holder: performer,
                    in_hand: true,
                },
            )
            .expect("Tried moving item");

        action::ok(format!(
            "{} picked up and wielded {}.",
            performer_name,
            item_name.definite()
        ))
    }
}

fn unwield_if_needed(world: &mut World, holder: Entity) {
    world
        .query_mut::<&mut Held>()
        .into_iter()
        .filter(|(_, held)| held.held_by(holder))
        .for_each(|(_, held)| held.in_hand = false);
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

pub fn drop_all_items(world: &mut World, entity: Entity) {
    let pos = *world.get::<&Pos>(entity).unwrap();
    let items = get_held(world, entity);
    for item in items {
        world.exchange_one::<Held, _>(item, pos).unwrap();
    }
}
