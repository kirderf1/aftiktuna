use crate::action::combat::Target;
use crate::position::Pos;
use crate::status;
use crate::view::{DisplayInfo, Messages};
use hecs::{Entity, World};
use rand::Rng;
use Action::*;

pub mod combat;
pub mod door;
pub mod item;
mod launch;
pub mod trade;

#[derive(Debug)]
pub struct CrewMember(pub Entity);

#[derive(Clone)]
pub enum Action {
    TakeItem(Entity, String),
    TakeAll,
    GiveItem(Entity, Entity),
    Wield(Entity, String),
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Entity),
    AttackNearest(Target),
    Wait,
    Rest(bool),
    Launch,
    Trade(Entity),
    Buy,
    ExitTrade,
}

pub fn tick(world: &mut World, rng: &mut impl Rng, messages: &mut Messages, aftik: Entity) {
    let mut entities = world
        .query::<&status::Stats>()
        .with::<&Action>()
        .iter()
        .map(|(entity, stats)| (entity, stats.agility))
        .collect::<Vec<_>>();
    entities.sort_by(|(_, agility1), (_, agility2)| agility2.cmp(agility1));
    let entities = entities
        .iter()
        .map(|(entity, _)| *entity)
        .collect::<Vec<_>>();

    for entity in entities {
        if !status::is_alive(entity, world) {
            continue;
        }

        if let Ok(action) = world.remove_one::<Action>(entity) {
            perform(world, rng, entity, action, aftik, messages);
        }
    }
}

fn perform(
    world: &mut World,
    rng: &mut impl Rng,
    performer: Entity,
    action: Action,
    controlled: Entity,
    messages: &mut Messages,
) {
    let result = match action {
        TakeItem(item, name) => item::take_item(world, performer, item, &name).map(Some),
        TakeAll => item::take_all(world, performer).map(Some),
        GiveItem(item, receiver) => item::give_item(world, performer, item, receiver).map(Some),
        Wield(item, name) => item::wield(world, performer, item, &name).map(Some),
        EnterDoor(door) => door::enter_door(world, performer, door).map(Some),
        ForceDoor(door) => door::force_door(world, performer, door).map(Some),
        Attack(target) => combat::attack(world, rng, performer, target).map(Some),
        AttackNearest(target) => combat::attack_nearest(world, rng, performer, target),
        Wait => Ok(None),
        Rest(first) => Ok(rest(world, performer, first)),
        Launch => Ok(launch::perform(world, performer)),
        Trade(shopkeeper) => trade::trade(world, performer, shopkeeper).map(Some),
        Buy => trade::buy(world, performer).map(Some),
        ExitTrade => trade::exit(world, performer).map(Some),
    };
    match result {
        Ok(Some(message)) => {
            let performer_pos = *world.get::<&Pos>(performer).unwrap();
            let player_pos = *world.get::<&Pos>(controlled).unwrap();
            if player_pos.is_in(performer_pos.get_area()) {
                messages.add(message);
            }
        }
        Ok(None) => {}
        Err(message) => {
            if performer == controlled {
                messages.add(message);
            }
        }
    }
}

fn rest(world: &mut World, performer: Entity, first: bool) -> Option<String> {
    let need_more_rest = world
        .get::<&status::Stamina>(performer)
        .map(|stamina| stamina.need_more_rest())
        .unwrap_or(false);

    if need_more_rest {
        world.insert_one(performer, Rest(false)).unwrap();
    }

    if first {
        Some(format!(
            "{} takes some time to rest up.",
            DisplayInfo::find_definite_name(world, performer)
        ))
    } else {
        None
    }
}
