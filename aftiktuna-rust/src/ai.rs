use crate::action::item::UseAction;
use crate::action::{Action, ForceDoorAction, TalkAction};
use crate::asset::species::SpeciesDataMap;
use crate::asset::{GameAssets, ItemTypeData};
use crate::core::area::{self, ShipControls, ShipState, ShipStatus};
use crate::core::behavior::{
    self, BadlyHurtBehavior, Character, GivesHuntRewardData, Hostile, Intention, ObservationTarget,
    RepeatingAction, Waiting, Wandering,
};
use crate::core::combat::{self, AttackKind};
use crate::core::item::ItemTypeId;
use crate::core::name::NameData;
use crate::core::position::{self, OccupiesSpace, Pos};
use crate::core::{CrewMember, Door, SpeciesId, Tag, inventory, status};
use crate::dialogue::TalkTopic;
use crate::game_loop::GameState;
use hecs::{CommandBuffer, Entity, EntityRef, Or, World};
use rand::Rng;
use rand::seq::{IndexedRandom, IteratorRandom};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

/// Prepares data for character behavior before the decision to take player action input.
pub fn prepare_intentions(state: &mut GameState, assets: &GameAssets) {
    let mut buffer = CommandBuffer::new();

    for (entity, occupies_space) in state.world.query::<&mut OccupiesSpace>().iter() {
        occupies_space.blocks_opponent = !has_behavior(
            state.world.entity(entity).unwrap(),
            BadlyHurtBehavior::Fearful,
            &assets.species_data_map,
        );
    }

    for (crew_member, _) in state
        .world
        .query::<()>()
        .with::<(&CrewMember, &Character)>()
        .iter()
    {
        if let Some(intention) = pick_intention(crew_member, state, assets) {
            buffer.insert_one(crew_member, intention);
        };
    }

    for (crew_member, action) in state
        .world
        .query::<&RepeatingAction>()
        .with::<&CrewMember>()
        .iter()
    {
        if action.cancel_if_unsafe()
            && let Ok(pos) = state.world.get::<&Pos>(crew_member)
            && !behavior::is_safe(&state.world, pos.get_area())
        {
            buffer.remove_one::<RepeatingAction>(crew_member);
        };
    }

    buffer.run_on(&mut state.world);
}

fn pick_intention(
    crew_member: Entity,
    state: &GameState,
    assets: &GameAssets,
) -> Option<Intention> {
    let world = &state.world;
    if world
        .get::<&status::Health>(crew_member)
        .is_ok_and(|health| health.is_badly_hurt())
        && let Ok(pos) = state.world.get::<&Pos>(crew_member)
        && behavior::is_safe(world, pos.get_area())
    {
        for item in inventory::get_inventory(world, crew_member) {
            if world
                .get::<&ItemTypeId>(item)
                .ok()
                .and_then(|id| assets.item_type_map.get(&id))
                .is_some_and(ItemTypeData::is_medkit)
            {
                return Some(Intention::UseMedkit(item));
            }
        }
    }

    let current_properties = combat::get_active_weapon_properties(world, crew_member, assets);

    for item in inventory::get_inventory(world, crew_member) {
        if let Some(properties) = world
            .get::<&ItemTypeId>(item)
            .ok()
            .and_then(|item_type| assets.item_type_map.get(&item_type))
            .and_then(|data| data.weapon)
            && properties.damage_mod > current_properties.damage_mod
        {
            return Some(Intention::Wield(item));
        }
    }

    let area = world.get::<&Pos>(crew_member).unwrap().get_area();
    if area::is_ship(area, world)
        && world
            .get::<&ShipState>(state.ship_core)
            .is_ok_and(|ship_state| matches!(ship_state.status, ShipStatus::NeedFuel(_)))
        && inventory::get_inventory(world, crew_member)
            .into_iter()
            .any(|item| {
                world
                    .get::<&ItemTypeId>(item)
                    .is_ok_and(|item_type| item_type.is_fuel_can())
            })
        && world
            .query::<&Pos>()
            .with::<&ShipControls>()
            .iter()
            .any(|(_, pos)| pos.is_in(area))
    {
        return Some(Intention::Refuel);
    }

    None
}

pub(crate) fn controlled_character_action(state: &GameState) -> Option<Action> {
    if state
        .world
        .satisfies::<&RepeatingAction>(state.controlled)
        .unwrap()
        || !behavior::is_safe(
            &state.world,
            state
                .world
                .get::<&Pos>(state.controlled)
                .unwrap()
                .get_area(),
        )
    {
        return None;
    }

    let area = state
        .world
        .get::<&Pos>(state.controlled)
        .unwrap()
        .get_area();
    if let Some((target, _)) = state
        .world
        .query::<(&Pos, &GivesHuntRewardData)>()
        .iter()
        .find(|(_, (pos, gives_hunt_reward))| {
            pos.is_in(area)
                && gives_hunt_reward.presented
                && gives_hunt_reward.is_fulfilled(&state.world)
        })
    {
        Some(
            TalkAction {
                target,
                topic: TalkTopic::CompleteHuntQuest,
            }
            .into(),
        )
    } else if is_wait_requested(&state.world, state.controlled) {
        Some(Action::Wait)
    } else {
        None
    }
}

fn is_wait_requested(world: &World, controlled: Entity) -> bool {
    let area = world.get::<&Pos>(controlled).unwrap().get_area();
    world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(entity, pos)| *entity != controlled && pos.is_in(area))
        .any(|(entity, _)| is_requesting_wait(world, entity))
}

pub fn tick(
    action_map: &mut HashMap<Entity, Action>,
    world: &mut World,
    rng: &mut impl Rng,
    assets: &GameAssets,
) {
    let mut buffer = CommandBuffer::new();

    for (entity, _) in world
        .query::<()>()
        .with::<Or<&CrewMember, &Hostile>>()
        .iter()
    {
        let entity_ref = world.entity(entity).unwrap();
        if status::is_alive_ref(entity_ref) && !action_map.contains_key(&entity) {
            let action = if let Some(action) = entity_ref.get::<&RepeatingAction>() {
                buffer.remove_one::<RepeatingAction>(entity);
                Action::from(*action)
            } else {
                pick_action(entity_ref, world, rng, assets).unwrap_or(Action::Wait)
            };

            action_map.insert(entity, action);
        };
    }

    world
        .query::<()>()
        .with::<&Intention>()
        .iter()
        .for_each(|(entity, _)| buffer.remove_one::<Intention>(entity));

    buffer.run_on(world);
}

fn pick_action(
    entity_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
    assets: &GameAssets,
) -> Option<Action> {
    if let Some(hostile) = entity_ref.get::<&Hostile>() {
        pick_foe_action(entity_ref, &hostile, world, rng, assets)
    } else if entity_ref.satisfies::<&CrewMember>() {
        pick_crew_action(entity_ref, world, rng, assets)
    } else {
        None
    }
}

fn pick_foe_action(
    entity_ref: EntityRef,
    hostile: &Hostile,
    world: &World,
    rng: &mut impl Rng,
    assets: &GameAssets,
) -> Option<Action> {
    if has_behavior(
        entity_ref,
        BadlyHurtBehavior::Fearful,
        &assets.species_data_map,
    ) && let Some(path) = find_random_unblocked_path(entity_ref, world, rng, |_| true)
    {
        return Some(Action::EnterDoor(path));
    }

    if hostile.aggressive {
        let area = entity_ref.get::<&Pos>()?.get_area();

        let targets = world
            .query::<&Pos>()
            .with::<&CrewMember>()
            .iter()
            .filter(|&(crew, crew_pos)| crew_pos.is_in(area) && status::is_alive(crew, world))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        if !targets.is_empty() {
            return Some(Action::Attack(
                targets,
                pick_attack_kind(entity_ref, world, rng, assets),
            ));
        }
    }

    if let Some(wandering) = entity_ref.get::<&Wandering>() {
        let area = entity_ref.get::<&Pos>()?.get_area();

        let observation_targets = world
            .query::<&Pos>()
            .with::<&ObservationTarget>()
            .iter()
            .filter(|&(_, pos)| pos.is_in(area))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        if let Some(&observation_target) = observation_targets.choose(rng)
            && rng.random_bool(8. / 10.)
        {
            return Some(Action::Examine(observation_target));
        } else if rng.random_bool(1. / 2.)
            && let Some(door) =
                find_random_unblocked_path(entity_ref, world, rng, |destination_area| {
                    wandering.area_tag.as_ref().is_none_or(|area_tag| {
                        world
                            .get::<&Tag>(destination_area)
                            .is_ok_and(|destination_tag| destination_tag.deref() == area_tag)
                    })
                })
        {
            return Some(Action::EnterDoor(door));
        }
    }

    None
}

fn pick_crew_action(
    entity_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
    assets: &GameAssets,
) -> Option<Action> {
    let entity_pos = *entity_ref.get::<&Pos>()?;

    if has_behavior(
        entity_ref,
        BadlyHurtBehavior::Fearful,
        &assets.species_data_map,
    ) {
        let is_area_safe =
            area::is_in_ship(entity_pos, world) && behavior::is_safe(world, entity_pos.get_area());
        if !is_area_safe {
            if let Some(path) = find_path_towards(world, entity_pos.get_area(), |area| {
                area::is_ship(area, world)
            }) && position::check_is_blocked(
                world,
                entity_ref,
                entity_pos,
                *world.get::<&Pos>(path).unwrap(),
            )
            .is_ok()
            {
                return Some(Action::EnterDoor(path));
            } else if let Some(path) = find_random_unblocked_path(entity_ref, world, rng, |_| true)
            {
                return Some(Action::EnterDoor(path));
            }
        }
    }

    let intention = entity_ref.get::<&Intention>();
    if let Some(&Intention::UseMedkit(item)) = intention.as_deref() {
        return Some(UseAction { item, use_time: 0 }.into());
    }

    if entity_ref
        .get::<&Waiting>()
        .is_some_and(|waiting| waiting.at_ship)
        && !area::is_in_ship(entity_pos, world)
        && let Some(path) = find_path_towards(world, entity_pos.get_area(), |area| {
            area::is_ship(area, world)
        })
        && position::check_is_blocked(
            world,
            entity_ref,
            entity_pos,
            *world.get::<&Pos>(path).unwrap(),
        )
        .is_ok()
    {
        return Some(Action::EnterDoor(path));
    }

    let foes = world
        .query::<(&Pos, &Hostile)>()
        .iter()
        .filter(|&(foe, (foe_pos, hostile))| {
            foe_pos.is_in(entity_pos.get_area())
                && status::is_alive(foe, world)
                && hostile.aggressive
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if !foes.is_empty() {
        return Some(Action::Attack(
            foes,
            pick_attack_kind(entity_ref, world, rng, assets),
        ));
    }

    if let Some(intention) = intention {
        match *intention {
            Intention::Wield(item) => {
                return Some(Action::Wield(item, NameData::find(world, item, assets)));
            }
            Intention::Force { door, assisted } => {
                return Some(
                    ForceDoorAction {
                        door,
                        assisting: Some(assisted),
                    }
                    .into(),
                );
            }
            Intention::Refuel => return Some(Action::Refuel),
            _ => {}
        };
    }

    None
}

pub fn pick_attack_kind(
    attacker_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
    assets: &GameAssets,
) -> AttackKind {
    let available_kinds =
        combat::get_active_weapon_properties(world, attacker_ref.entity(), assets)
            .attack_set
            .available_kinds();

    if has_behavior(
        attacker_ref,
        BadlyHurtBehavior::Determined,
        &assets.species_data_map,
    ) && available_kinds.contains(&AttackKind::Rash)
    {
        AttackKind::Rash
    } else {
        available_kinds
            .choose(rng)
            .copied()
            .unwrap_or(AttackKind::Light)
    }
}

pub fn is_requesting_wait(world: &World, entity: Entity) -> bool {
    world
        .satisfies::<hecs::Or<hecs::Or<&Intention, &RepeatingAction>, &status::IsStunned>>(entity)
        .unwrap_or(false)
}

fn find_random_unblocked_path(
    entity_ref: EntityRef,
    world: &World,
    rng: &mut impl Rng,
    destination_area_filter: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    let entity_pos = *entity_ref.get::<&Pos>()?;
    world
        .query::<(&Pos, &Door)>()
        .iter()
        .filter(|&(_, (path_pos, door))| {
            path_pos.is_in(entity_pos.get_area())
                && position::check_is_blocked(world, entity_ref, entity_pos, *path_pos).is_ok()
                && destination_area_filter(door.destination.get_area())
        })
        .choose(rng)
        .map(|(path, _)| path)
}

fn has_behavior(
    entity_ref: EntityRef,
    behavior: BadlyHurtBehavior,
    species_map: &SpeciesDataMap,
) -> bool {
    entity_ref
        .get::<&status::Health>()
        .is_some_and(|health| health.is_badly_hurt())
        && entity_ref
            .get::<&SpeciesId>()
            .and_then(|species_id| species_map.get(&species_id))
            .and_then(|species_data| species_data.badly_hurt_behavior)
            == Some(behavior)
}

struct PathSearchEntry {
    path: Entity,
    area: Entity,
}

impl PathSearchEntry {
    fn start(path_entity: Entity, path: &Door) -> Self {
        Self {
            path: path_entity,
            area: path.destination.get_area(),
        }
    }

    fn next(&self, path: &Door) -> Self {
        Self {
            path: self.path,
            area: path.destination.get_area(),
        }
    }
}

pub fn find_path_towards(
    world: &World,
    area: Entity,
    predicate: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    let mut entries = world
        .query::<(&Pos, &Door)>()
        .iter()
        .filter(|&(_, (pos, _))| pos.is_in(area))
        .map(|(entity, (_, path))| PathSearchEntry::start(entity, path))
        .collect::<Vec<_>>();
    let mut checked_areas = HashSet::from([area]);

    while !entries.is_empty() {
        let mut new_entries = vec![];
        for entry in entries {
            if checked_areas.insert(entry.area) {
                if predicate(entry.area) {
                    return Some(entry.path);
                }
                new_entries.extend(
                    world
                        .query::<(&Pos, &Door)>()
                        .iter()
                        .filter(|&(_, (pos, _))| pos.is_in(entry.area))
                        .map(|(_, (_, path))| entry.next(path)),
                );
            }
        }
        entries = new_entries;
    }

    None
}
