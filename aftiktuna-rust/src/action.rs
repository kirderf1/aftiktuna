use crate::action::combat::Target;
use crate::position::{try_move_adjacent, Pos};
use crate::view::{Messages, NameData, TextureType};
use crate::{status, view};
use hecs::{Entity, World};
use rand::Rng;
use std::result;
use Action::*;

pub mod combat;
pub mod door;
pub mod item;
mod launch;
pub mod trade;

#[derive(Debug)]
pub struct CrewMember(pub Entity);

pub struct Recruitable(pub String);

#[derive(Clone)]
pub enum Action {
    TakeItem(Entity, NameData),
    TakeAll,
    GiveItem(Entity, Entity),
    Wield(Entity, NameData),
    UseMedkit(Entity),
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Entity),
    AttackNearest(Target),
    Wait,
    Rest(bool),
    Launch,
    Recruit(Entity),
    Trade(Entity),
    Buy(crate::item::Type, u16),
    Sell(Vec<Entity>),
    ExitTrade,
    OpenChest(Entity),
}

pub fn tick(
    world: &mut World,
    rng: &mut impl Rng,
    messages: &mut Messages,
    aftik: Entity,
    is_at_fortuna: bool,
) {
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
            perform(world, rng, entity, action, aftik, messages, is_at_fortuna);
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
    is_at_fortuna: bool,
) {
    let result = match action {
        OpenChest(chest) => open_chest(world, performer, chest),
        TakeItem(item, name) => item::take_item(world, performer, item, name),
        TakeAll => item::take_all(world, performer),
        GiveItem(item, receiver) => item::give_item(world, performer, item, receiver),
        Wield(item, name) => item::wield(world, performer, item, name),
        UseMedkit(item) => item::use_medkit(world, performer, item),
        EnterDoor(door) => door::enter_door(world, performer, door),
        ForceDoor(door) => door::force_door(world, performer, door),
        Attack(target) => combat::attack(world, rng, performer, target),
        AttackNearest(target) => combat::attack_nearest(world, rng, performer, target),
        Wait => silent_ok(),
        Rest(first) => rest(world, performer, first),
        Launch => launch::perform(world, performer, is_at_fortuna),
        Recruit(target) => recruit(world, performer, target),
        Trade(shopkeeper) => trade::trade(world, performer, shopkeeper),
        Buy(item_type, amount) => trade::buy(world, performer, item_type, amount),
        Sell(items) => trade::sell(world, performer, items),
        ExitTrade => trade::exit(world, performer),
    };
    match result {
        Ok(Success::LocalMessage(message)) => {
            let performer_pos = *world.get::<&Pos>(performer).unwrap();
            let player_pos = *world.get::<&Pos>(controlled).unwrap();
            if player_pos.is_in(performer_pos.get_area()) {
                messages.add(message);
            }
        }
        Ok(Success::Message(message, areas)) => {
            let player_pos = *world.get::<&Pos>(controlled).unwrap();
            if areas.contains(&player_pos.get_area()) {
                messages.add(message);
            }
        }
        Ok(Success::Silent) => {}
        Err(message) => {
            if performer == controlled {
                messages.add(message);
            }
        }
    }
}

fn rest(world: &mut World, performer: Entity, first_turn_resting: bool) -> Result {
    let area = world.get::<&Pos>(performer).unwrap().get_area();

    let need_more_rest = world
        .query::<(&status::Stamina, &Pos)>()
        .with::<&CrewMember>()
        .iter()
        .any(|(_, (stamina, pos))| pos.is_in(area) && stamina.need_more_rest());

    if need_more_rest {
        world.insert_one(performer, Rest(false)).unwrap();
    }

    if first_turn_resting {
        ok("The crew takes some time to rest up.".to_string())
    } else {
        silent_ok()
    }
}

fn recruit(world: &mut World, performer: Entity, target: Entity) -> Result {
    let target_pos = *world.get::<&Pos>(target).unwrap();
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let crew_size = world.query::<&CrewMember>().iter().count();
    if crew_size >= 2 {
        return Err("There is not enough room for another crew member.".to_string());
    }

    try_move_adjacent(world, performer, target_pos)?;
    let Recruitable(name) = world.remove_one::<Recruitable>(target).unwrap();
    world
        .insert(
            target,
            (
                view::name_display_info(TextureType::Aftik, &name),
                NameData::Name(name.clone()),
                CrewMember(crew),
            ),
        )
        .unwrap();
    ok(format!("{} joined the crew!", name))
}

pub struct FortunaChest;

pub struct OpenedChest;

fn open_chest(world: &mut World, performer: Entity, chest: Entity) -> Result {
    let chest_pos = *world.get::<&Pos>(chest).unwrap();

    try_move_adjacent(world, performer, chest_pos)?;

    if world.get::<&FortunaChest>(chest).is_err() {
        return Err(format!(
            "{} tried to open {}, but that is not the fortuna chest!",
            NameData::find(world, performer).definite(),
            NameData::find(world, chest).definite()
        ));
    }

    world.insert_one(performer, OpenedChest).unwrap();
    ok(format!(
        "{} opened the fortuna chest and found the item that they desired the most.",
        NameData::find(world, performer).definite()
    ))
}

type Result = result::Result<Success, String>;

pub enum Success {
    LocalMessage(String),
    Message(String, Vec<Entity>),
    Silent,
}

fn ok(message: String) -> Result {
    Ok(Success::LocalMessage(message))
}

fn ok_at(message: String, areas: Vec<Entity>) -> Result {
    Ok(Success::Message(message, areas))
}

fn silent_ok() -> Result {
    Ok(Success::Silent)
}
