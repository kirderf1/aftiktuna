use crate::action;
use crate::action::Action;
use crate::core::inventory::Held;
use crate::core::item::{Item, Medkit};
use crate::core::name::{NameData, NameQuery};
use crate::core::position::{Blockage, Pos};
use crate::core::status::Health;
use crate::core::{self, inventory, position, status};
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
        .query::<&Pos>()
        .with::<NameQuery>()
        .with::<&Item>()
        .iter()
        .any(|(_, pos)| pos.is_in(aftik_pos.get_area()))
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

    position::move_to(world, performer, item_pos)?;
    world
        .exchange_one::<Pos, _>(item, Held::in_inventory(performer))
        .expect("Tried moving item to inventory");

    core::trigger_aggression_in_area(world, item_pos.get_area());

    action::ok(format!(
        "{} picked up {}.",
        performer_name,
        item_name.definite()
    ))
}

pub(super) fn give_item(
    mut context: super::Context,
    performer: Entity,
    item: Entity,
    receiver: Entity,
) -> action::Result {
    let world = context.mut_world();
    let performer_name = NameData::find(world, performer).definite();
    let receiver_name = NameData::find(world, receiver).definite();

    if world
        .get::<&Held>(item)
        .ok()
        .filter(|in_inv| in_inv.held_by(performer))
        .is_none()
    {
        return Err(format!(
            "{performer_name} lost track of the item they were going to give."
        ));
    }

    let performer_pos = *world
        .get::<&Pos>(performer)
        .expect("Expected performer to have a position");
    let receiver_pos = *world.get::<&Pos>(receiver).map_err(|_| {
        format!("{receiver_name} disappeared before {performer_name} could interact with them.",)
    })?;

    if !performer_pos.is_in(receiver_pos.get_area()) {
        return Err(format!(
            "{receiver_name} left before {performer_name} could interact with them.",
        ));
    }

    if !status::is_alive(receiver, world) {
        return Err(format!(
            "{receiver_name} died before they could be given an item."
        ));
    }

    let movement = position::prepare_move_adjacent(world, performer, receiver_pos)
        .map_err(Blockage::into_message)?;

    context.capture_frame_for_dialogue();

    movement.perform(context.mut_world()).unwrap();

    context.add_dialogue(performer, "\"Here, hold on to this.\"");

    let world = context.mut_world();
    world
        .insert_one(item, Held::in_inventory(receiver))
        .unwrap();

    super::ok(format!(
        "{performer_name} gave {receiver_name} a {}.",
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
        position::move_to(world, performer, item_pos)?;

        inventory::unwield_if_needed(world, performer);
        world
            .exchange_one::<Pos, _>(item, Held::in_hand(performer))
            .expect("Tried moving item");

        core::trigger_aggression_in_area(world, item_pos.get_area());

        action::ok(format!(
            "{performer_name} picked up and wielded {}.",
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
