use crate::core::item::Type as ItemType;
use crate::core::position::{try_move_adjacent, Pos};
use crate::core::{status, GameState};
use crate::view;
use crate::view::{NameData, TextureType};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::result;
use Action::*;

pub mod combat;
pub mod door;
pub mod item;
mod launch;
pub mod trade;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Serialize, Deserialize)]
pub struct Recruitable(pub String);

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    TakeItem(Entity, NameData),
    TakeAll,
    GiveItem(Entity, Entity),
    Wield(Entity, NameData),
    UseMedkit(Entity),
    EnterDoor(Entity),
    ForceDoor(Entity),
    Attack(Vec<Entity>),
    Wait,
    Rest(bool),
    Launch,
    Recruit(Entity),
    Trade(Entity),
    Buy(ItemType, u16),
    Sell(Vec<Entity>),
    ExitTrade,
    OpenChest(Entity),
}

pub fn tick(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let mut entities = state
        .world
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
        if !status::is_alive(entity, &state.world) {
            continue;
        }

        if let Ok(action) = state.world.remove_one::<Action>(entity) {
            perform(state, entity, action, view_buffer);
        }
    }
}

fn perform(
    state: &mut GameState,
    performer: Entity,
    action: Action,
    view_buffer: &mut view::Buffer,
) {
    let result = match action {
        OpenChest(chest) => open_chest(&mut state.world, performer, chest),
        TakeItem(item, name) => item::take_item(&mut state.world, performer, item, name),
        TakeAll => item::take_all(&mut state.world, performer),
        GiveItem(item, receiver) => item::give_item(&mut state.world, performer, item, receiver),
        Wield(item, name) => item::wield(&mut state.world, performer, item, name),
        UseMedkit(item) => item::use_medkit(&mut state.world, performer, item),
        EnterDoor(door) => door::enter_door(&mut state.world, performer, door),
        ForceDoor(door) => door::force_door(&mut state.world, performer, door),
        Attack(targets) => combat::attack(state, performer, targets),
        Wait => silent_ok(),
        Rest(first) => rest(&mut state.world, performer, first),
        Launch => launch::perform(state, performer),
        Recruit(target) => recruit(&mut state.world, performer, target),
        Trade(shopkeeper) => trade::trade(&mut state.world, performer, shopkeeper),
        Buy(item_type, amount) => trade::buy(&mut state.world, performer, item_type, amount),
        Sell(items) => trade::sell(&mut state.world, performer, items),
        ExitTrade => trade::exit(&mut state.world, performer),
    };

    let world = &state.world;
    let controlled = state.controlled;
    match result {
        Ok(Success::LocalMessage(message)) => {
            let performer_pos = *world.get::<&Pos>(performer).unwrap();
            let player_pos = *world.get::<&Pos>(controlled).unwrap();
            if player_pos.is_in(performer_pos.get_area()) {
                view_buffer.messages.add(message);
            }
        }
        Ok(Success::Message(message, areas)) => {
            let player_pos = *world.get::<&Pos>(controlled).unwrap();
            if areas.contains(&player_pos.get_area()) {
                view_buffer.messages.add(message);
            }
        }
        Ok(Success::Silent) => {}
        Err(message) => {
            if performer == controlled {
                view_buffer.messages.add(message);
                view_buffer.capture_view(state);
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

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
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
