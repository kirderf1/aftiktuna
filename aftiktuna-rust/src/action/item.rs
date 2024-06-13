use crate::action::{self, Error};
use crate::core::inventory::Held;
use crate::core::item::{FourLeafClover, Item, Medkit, Usable};
use crate::core::name::{NameData, NameQuery};
use crate::core::position::Pos;
use crate::core::status::{Health, StatChanges};
use crate::core::{self, inventory, position, status, RepeatingAction};
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
        world.insert_one(aftik, RepeatingAction::TakeAll).unwrap();
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

    position::push_and_move(world, performer, item_pos)?;

    if world.satisfies::<&FourLeafClover>(item).unwrap()
        && FOUR_LEAF_CLOVER_EFFECT
            .try_apply(world.entity(performer).unwrap())
            .is_some()
    {
        world.despawn(item).unwrap();

        return action::ok(format!(
            "{performer_name} tries to pick up {}. But as they do, it disappears in their hand. (Luck has increased by 2 points)",
            item_name.definite(),
        ));
    }

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

#[derive(Debug, Clone)]
pub struct SearchAction {
    pub container: Entity,
}

impl From<SearchAction> for super::Action {
    fn from(value: SearchAction) -> Self {
        Self::Search(value)
    }
}

impl SearchAction {
    pub(super) fn run(self, performer: Entity, mut context: super::Context) -> action::Result {
        let Self { container } = self;
        let world = context.mut_world();
        let performer_name = NameData::find(world, performer).definite();
        let container_name = NameData::find(world, container).definite();
        let container_pos = *world
            .get::<&Pos>(container)
            .map_err(|_| format!("{performer_name} lost track of {container_name}."))?;

        if !world
            .satisfies::<&inventory::Container>(container)
            .unwrap_or(false)
        {
            return Err(Error::private(format!(
                "{container_name} is not a searchable container."
            )));
        }

        position::push_and_move(world, performer, container_pos)?;

        let items = inventory::get_held(world, container);
        if items.is_empty() {
            return action::ok(format!("{performer_name} searched {container_name}, but did not find anything of interest."));
        }

        inventory::drop_all_items(world, container);

        core::trigger_aggression_in_area(world, container_pos.get_area());

        let items = items
            .into_iter()
            .map(|item| NameData::find(world, item).base().to_owned())
            .collect::<Vec<_>>()
            .join(" and a ");
        action::ok(format!(
            "{performer_name} searched {container_name} and found a {items}."
        ))
    }
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
        return Err(Error::private(format!(
            "{performer_name} lost track of the item they were going to give."
        )));
    }

    let performer_pos = *world
        .get::<&Pos>(performer)
        .expect("Expected performer to have a position");
    let receiver_pos = *world.get::<&Pos>(receiver).map_err(|_| {
        format!("{receiver_name} disappeared before {performer_name} could interact with them.",)
    })?;

    if !performer_pos.is_in(receiver_pos.get_area()) {
        return Err(Error::private(format!(
            "{receiver_name} left before {performer_name} could interact with them.",
        )));
    }

    if !status::is_alive(receiver, world) {
        return Err(Error::private(format!(
            "{receiver_name} died before they could be given an item."
        )));
    }

    let movement = position::prepare_move_adjacent(world, performer, receiver_pos)
        .map_err(|blockage| blockage.into_message(world))?;

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
        position::push_and_move(world, performer, item_pos)?;

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

#[derive(Debug, Clone)]
pub struct UseAction {
    pub item: Entity,
}

impl From<UseAction> for super::Action {
    fn from(value: UseAction) -> Self {
        Self::Use(value)
    }
}

impl UseAction {
    pub(super) fn run(self, performer: Entity, mut context: super::Context) -> action::Result {
        let world = context.mut_world();

        let performer_ref = world.entity(performer).unwrap();
        let performer_name = NameData::find_by_ref(performer_ref).definite();

        let item_ref = world
            .entity(self.item)
            .ok()
            .filter(|item_ref| {
                item_ref
                    .get::<&Held>()
                    .map_or(false, |held| held.held_by(performer))
            })
            .ok_or_else(|| format!("{performer_name} tried using an item not held by them."))?;
        let item_name = NameData::find_by_ref(item_ref).definite();

        if item_ref.satisfies::<&Medkit>() {
            let mut health = performer_ref.get::<&mut Health>().unwrap();
            if !health.is_hurt() {
                return Err(Error::private(format!(
                    "{performer_name} no longer needs to use a medkit.",
                )));
            }

            health.restore_fraction(0.5);
            drop(health);
            world.despawn(self.item).unwrap();

            return action::ok(format!(
                "{performer_name} used a medkit and recovered some health.",
            ));
        }

        let Some(usable) = item_ref.get::<&Usable>().as_deref().copied() else {
            return Err(Error::private(format!(
                "{performer_name} tried to use {item_name}, but it is not usable."
            )));
        };

        match usable {
            Usable::BlackOrb => {
                let Some(_) = BLACK_ORB_EFFECT.try_apply(performer_ref) else {
                    return action::ok(format!("{performer_name} holds up and inspects the orb, but can't figure out what it is."));
                };

                world.despawn(self.item).unwrap();

                action::ok(format!(
                    "{performer_name} holds up and inspects the orb. \
                     {performer_name} gets a sensation of hardiness when suddenly, \
                     the orb cracks and falls apart into worthless pieces! (Stats have changed)"
                ))
            }
        }
    }
}

const BLACK_ORB_EFFECT: StatChanges = StatChanges {
    endurance: 3,
    agility: -1,
    luck: -1,
    ..StatChanges::DEFAULT
};

pub const FOUR_LEAF_CLOVER_EFFECT: StatChanges = StatChanges {
    luck: 2,
    ..StatChanges::DEFAULT
};
