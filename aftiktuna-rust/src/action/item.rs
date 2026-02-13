use crate::action::{self, Context, Error};
use crate::asset::ItemUseType;
use crate::core::behavior::{self, RepeatingAction};
use crate::core::inventory::{self, Held};
use crate::core::item::ItemTypeId;
use crate::core::name::{self, ArticleKind, CountFormat, NameData, NameIdData, NameQuery};
use crate::core::position::{self, Placement, PlacementQuery, Pos};
use crate::core::status::{self, Health, StatChanges, Stats};
use crate::dialogue;
use crate::view::text::{self, CombinableMsgType};
use hecs::Entity;

pub(super) fn take_all(context: &mut Context, aftik: Entity) -> action::Result {
    let world = &mut context.state.world;
    let aftik_pos = *world.get::<&Pos>(aftik).unwrap();
    let (item, name) = world
        .query::<(&Pos, NameQuery)>()
        .with::<&ItemTypeId>()
        .iter()
        .filter(|(_, (pos, _))| pos.is_in(aftik_pos.get_area()))
        .min_by_key(|(_, (pos, _))| pos.distance_to(aftik_pos))
        .map(|(item, (_, query))| {
            (
                item,
                NameData::from_query(query, context.view_context.view_buffer.assets),
            )
        })
        .ok_or("There are no items to take here.")?;

    let result = take_item(context, aftik, item, name)?;

    if context
        .state
        .world
        .query::<&Pos>()
        .with::<NameQuery>()
        .with::<&ItemTypeId>()
        .iter()
        .any(|(_, pos)| pos.is_in(aftik_pos.get_area()))
    {
        context
            .state
            .world
            .insert_one(aftik, RepeatingAction::TakeAll)
            .unwrap();
    }
    Ok(result)
}

pub(super) fn take_item(
    context: &mut action::Context,
    performer: Entity,
    item: Entity,
    item_name: NameData,
) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let performer_name = NameIdData::find(&context.state.world, performer);
    let item_pos = *context.state.world.get::<&Pos>(item).map_err(|_| {
        format!(
            "{the_performer} lost track of {the_item}.",
            the_performer = performer_name.clone().lookup(assets).definite(),
            the_item = item_name.definite()
        )
    })?;
    let item_name = NameIdData::find(&context.state.world, item);

    context
        .view_context
        .capture_unseen_view(item_pos.get_area(), context.state);

    let world = &mut context.state.world;
    position::push_and_move(world, performer, item_pos, assets)?;

    if world
        .get::<&ItemTypeId>(item)
        .is_ok_and(|item_type| item_type.is_four_leaf_clover())
        && FOUR_LEAF_CLOVER_EFFECT
            .try_apply(world.entity(performer).unwrap())
            .is_some()
    {
        world.despawn(item).unwrap();

        context.view_context.add_message_at(item_pos.get_area(), format!(
            "{the_performer} tries to pick up {the_item}. But as they do, it disappears in their hand. (Luck has increased by 2 points)",
            the_performer = performer_name.clone().lookup(assets).definite(),
            the_item = item_name.lookup(assets).definite(),
        ), context.state);
        return Ok(action::Success);
    }

    world
        .exchange_one::<Pos, _>(item, Held::in_inventory(performer))
        .expect("Tried moving item to inventory");

    behavior::trigger_aggression_in_area(world, item_pos.get_area());

    context.view_context.add_message_at(
        item_pos.get_area(),
        CombinableMsgType::PickUp(performer_name).message(item_name),
        context.state,
    );
    Ok(action::Success)
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
    pub(super) fn run(self, performer: Entity, mut context: Context) -> action::Result {
        let assets = context.view_context.view_buffer.assets;
        let Self { container } = self;
        let world = context.mut_world();
        let performer_name = NameData::find(world, performer, assets).definite();
        let container_name = NameData::find(world, container, assets).definite();
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

        position::push_and_move(world, performer, container_pos, assets)?;

        let items = inventory::get_held(world, container);
        if items.is_empty() {
            context.view_context.add_message_at(container_pos.get_area(), format!(
                "{performer_name} searched {container_name}, but did not find anything of interest."
            ), context.state);
            return Ok(action::Success);
        }

        inventory::drop_all_items(world, container);

        behavior::trigger_aggression_in_area(world, container_pos.get_area());

        let items = name::names_with_counts(
            items.into_iter().map(|item| NameIdData::find(world, item)),
            ArticleKind::A,
            CountFormat::Text,
            assets,
        );
        context.view_context.add_message_at(
            container_pos.get_area(),
            format!(
                "{performer_name} searched {container_name} and found {items}.",
                items = text::join_elements(items)
            ),
            context.state,
        );
        Ok(action::Success)
    }
}

pub(super) fn give_item(
    context: Context,
    performer: Entity,
    item: Entity,
    receiver: Entity,
) -> action::Result {
    let Context {
        state,
        mut view_context,
    } = context;
    let assets = view_context.view_buffer.assets;
    let world = &mut state.world;
    let performer_name = NameData::find(world, performer, assets).definite();
    let receiver_name = NameData::find(world, receiver, assets).definite();

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
    let receiver_placement =
        world
            .query_one_mut::<PlacementQuery>(receiver)
            .map(Placement::from)
            .map_err(|_| {
                format!(
                    "{receiver_name} disappeared before {performer_name} could interact with them.",
                )
            })?;

    if !performer_pos.is_in(receiver_placement.area()) {
        return Err(Error::private(format!(
            "{receiver_name} left before {performer_name} could interact with them.",
        )));
    }

    if !status::is_alive(receiver, world) {
        return Err(Error::private(format!(
            "{receiver_name} died before they could be given an item."
        )));
    }

    let movement = position::prepare_move_adjacent_placement(world, performer, receiver_placement)
        .map_err(|blockage| blockage.into_message(world, assets))?;

    view_context.capture_frame_for_dialogue(state);

    movement.perform(&mut state.world).unwrap();

    dialogue::trigger_dialogue_by_name(
        "give_item",
        performer,
        receiver,
        state,
        view_context.view_buffer,
    );

    state
        .world
        .insert_one(item, Held::in_inventory(receiver))
        .unwrap();

    view_context.add_message_at(
        performer_pos.get_area(),
        format!(
            "{performer_name} gave {receiver_name} a {}.",
            NameData::find(&state.world, item, assets).base(),
        ),
        state,
    );
    Ok(action::Success)
}

pub(super) fn wield(
    context: &mut Context,
    performer: Entity,
    item: Entity,
    item_name: NameData,
) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let world = &mut context.state.world;
    let performer_name = NameData::find(world, performer, assets).definite();

    if inventory::is_in_inventory(world, item, performer) {
        inventory::unwield_if_needed(world, performer);
        world.insert_one(item, Held::in_hand(performer)).unwrap();

        let performer_area = world.get::<&Pos>(performer).unwrap().get_area();
        context.view_context.add_message_at(
            performer_area,
            format!("{performer_name} wielded {}.", item_name.definite()),
            context.state,
        );
        Ok(action::Success)
    } else {
        let item_pos = *world
            .get::<&Pos>(item)
            .map_err(|_| format!("{} lost track of {}.", performer_name, item_name.definite()))?;
        position::push_and_move(world, performer, item_pos, assets)?;

        inventory::unwield_if_needed(world, performer);
        world
            .exchange_one::<Pos, _>(item, Held::in_hand(performer))
            .expect("Tried moving item");

        behavior::trigger_aggression_in_area(world, item_pos.get_area());

        context.view_context.add_message_at(
            item_pos.get_area(),
            format!(
                "{performer_name} picked up and wielded {}.",
                item_name.definite()
            ),
            context.state,
        );
        Ok(action::Success)
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
    pub(super) fn run(self, performer: Entity, mut context: Context) -> action::Result {
        let assets = context.view_context.view_buffer.assets;
        let world = &mut context.state.world;

        let performer_ref = world.entity(performer).unwrap();
        let performer_name = NameData::find_by_ref(performer_ref, assets).definite();
        let area = performer_ref.get::<&Pos>().unwrap().get_area();

        let item_ref = world
            .entity(self.item)
            .ok()
            .filter(|item_ref| {
                item_ref
                    .get::<&Held>()
                    .is_some_and(|held| held.held_by(performer))
            })
            .ok_or_else(|| format!("{performer_name} tried using an item not held by them."))?;
        let item_name = NameData::find_by_ref(item_ref, assets).definite();

        let item_use_type = item_ref
            .get::<&ItemTypeId>()
            .and_then(|id| assets.item_type_map.get(&id))
            .and_then(|data| data.usage.as_ref())
            .ok_or_else(|| {
                Error::private(format!(
                    "{performer_name} tried to use {item_name}, but it is not usable."
                ))
            })?;

        match *item_use_type {
            ItemUseType::Medkit { restore_fraction } => {
                let mut health = performer_ref.get::<&mut Health>().unwrap();
                if !health.is_hurt() {
                    return Err(Error::private(format!(
                        "{performer_name} no longer needs to use a medkit.",
                    )));
                }

                health.restore_fraction(restore_fraction, performer_ref);
                drop(health);
                world.despawn(self.item).unwrap();

                context.view_context.add_message_at(
                    area,
                    format!("{performer_name} used a medkit and recovered some health."),
                    context.state,
                );
                Ok(action::Success)
            }
            ItemUseType::BlackOrb { change } => {
                let Some(_) = change.try_apply(performer_ref) else {
                    context.view_context.add_message_at(area, format!(
                        "{performer_name} holds up and inspects the orb, but can't figure out what it is."
                    ), context.state);
                    return Ok(action::Success);
                };

                world.despawn(self.item).unwrap();

                context.view_context.add_message_at(
                    area,
                    format!(
                        "{performer_name} holds up and inspects the orb. \
                     {performer_name} gets a sensation of hardiness when suddenly, \
                     the orb cracks and falls apart into worthless pieces! (Stats have changed)"
                    ),
                    context.state,
                );
                Ok(action::Success)
            }
            ItemUseType::OddHandMirror { sum_change } => {
                let Some(action::Success) =
                    performer_ref.get::<&mut Stats>().and_then(|mut stats| {
                        let target_sum = stats.sum() + sum_change;

                        use rand::seq::IteratorRandom;
                        let rng = &mut context.state.rng;
                        let mut random_stats = Stats {
                            strength: (1..=10).choose(rng).unwrap(),
                            endurance: (1..=10).choose(rng).unwrap(),
                            agility: (1..=10).choose(rng).unwrap(),
                            luck: (0..=10).choose(rng).unwrap(),
                        };
                        while random_stats.sum() < target_sum {
                            random_stats.adjust_random_in_bounds(1, rng).ok()?;
                        }
                        while random_stats.sum() > target_sum {
                            random_stats.adjust_random_in_bounds(-1, rng).ok()?;
                        }

                        *stats = random_stats;
                        Some(action::Success)
                    })
                else {
                    context.view_context.add_message_at(area, format!(
                        "{performer_name} gazes into the mirror, but nothing seems to come from doing so."
                    ), context.state);
                    return Ok(action::Success);
                };

                world.despawn(self.item).unwrap();

                context.view_context.add_message_at(
                    area,
                    format!(
                        "{performer_name} holds up and gazes into the mirror. After a few moments, the glass suddenly cracks! \
                     Turning away from the broken mirror, {performer_name} gives off a different vibe from just a moment ago. (Stats have changed)"
                    ),
                    context.state,
                );
                Ok(action::Success)
            }
        }
    }
}

pub const FOUR_LEAF_CLOVER_EFFECT: StatChanges = StatChanges {
    luck: 2,
    ..StatChanges::DEFAULT
};
