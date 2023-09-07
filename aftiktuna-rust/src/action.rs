use crate::core::item::Type as ItemType;
use crate::core::position::{Blockage, Pos};
use crate::core::{position, status, GameState};
use crate::view;
use crate::view::name::{Name, NameData};
use crate::view::{Frame, Symbol};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::result;
use Action::*;

pub mod combat;
pub mod door;
mod item;
mod launch;
pub mod trade;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Serialize, Deserialize)]
pub struct Recruitable;

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
    TalkTo(Entity),
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
    let context = Context { state, view_buffer };
    let result = match action {
        OpenChest(chest) => open_chest(&mut state.world, performer, chest),
        TakeItem(item, name) => item::take_item(&mut state.world, performer, item, name),
        TakeAll => item::take_all(&mut state.world, performer),
        GiveItem(item, receiver) => item::give_item(context, performer, item, receiver),
        Wield(item, name) => item::wield(&mut state.world, performer, item, name),
        UseMedkit(item) => item::use_medkit(&mut state.world, performer, item),
        EnterDoor(door) => door::enter_door(&mut state.world, performer, door),
        ForceDoor(door) => door::force_door(&mut state.world, performer, door),
        Attack(targets) => combat::attack(state, performer, targets),
        Wait => silent_ok(),
        Rest(first) => rest(&mut state.world, performer, first),
        Launch => launch::perform(state, performer),
        TalkTo(target) => talk_to(context, performer, target),
        Recruit(target) => recruit(context, performer, target),
        Trade(shopkeeper) => trade::trade(&mut state.world, performer, shopkeeper),
        Buy(item_type, amount) => trade::buy(&mut state.world, performer, item_type, amount),
        Sell(items) => trade::sell(&mut state.world, performer, items),
        ExitTrade => trade::exit(&mut state.world, performer),
    };

    let world = &state.world;
    let controlled = state.controlled;
    match result {
        Ok(Success { message: None, .. }) => {}
        Ok(Success {
            message: Some(message),
            areas,
        }) => {
            let areas =
                areas.unwrap_or_else(|| vec![world.get::<&Pos>(performer).unwrap().get_area()]);
            let player_pos = *world.get::<&Pos>(controlled).unwrap();
            if areas.contains(&player_pos.get_area()) {
                view_buffer.messages.add(message);
            }
        }
        Err(message) => {
            if performer == controlled {
                view_buffer.messages.add(message);
                view_buffer.capture_view(state);
            }
        }
    }
}

struct Context<'a> {
    state: &'a mut GameState,
    view_buffer: &'a mut view::Buffer,
}

impl<'a> Context<'a> {
    fn mut_world(&mut self) -> &mut World {
        &mut self.state.world
    }
    fn add_dialogue(&mut self, frames: Vec<Frame>) {
        if !self.view_buffer.messages.0.is_empty() {
            self.view_buffer.capture_view(self.state);
        }
        for frame in frames {
            self.view_buffer.push_frame(frame);
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

fn talk_to(mut context: Context, performer: Entity, target: Entity) -> Result {
    let world = context.mut_world();
    if !status::is_alive(target, world) {
        return silent_ok();
    }
    let target_pos = *world.get::<&Pos>(target).unwrap();

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(Blockage::into_message)?;

    let frames = vec![
        Frame::new_dialogue(world, performer, vec!["\"Hi!\"".to_owned()]),
        Frame::new_dialogue(world, target, vec!["\"Hello!\"".to_owned()]),
    ];
    context.add_dialogue(frames);

    let world = context.mut_world();
    movement.perform(world).unwrap();

    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

fn recruit(mut context: Context, performer: Entity, target: Entity) -> Result {
    let world = context.mut_world();
    let target_pos = *world.get::<&Pos>(target).unwrap();
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let crew_size = world.query::<&CrewMember>().iter().count();
    if crew_size >= 2 {
        return Err("There is not enough room for another crew member.".to_string());
    }

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(Blockage::into_message)?;

    let frames = vec![
        Frame::new_dialogue(
            world,
            performer,
            vec!["\"Hi! Do you want to join me in the search for Fortuna?\"".to_owned()],
        ),
        Frame::new_dialogue(world, target, vec!["\"Sure, I'll join you!\"".to_owned()]),
    ];
    context.add_dialogue(frames);

    let world = context.mut_world();
    movement.perform(world).unwrap();
    world.remove_one::<Recruitable>(target).unwrap();
    if let Ok(mut name) = world.get::<&mut Name>(target) {
        name.set_is_known();
    }
    let name = NameData::find(world, target).definite();
    world
        .insert(target, (Symbol::from_name(&name), CrewMember(crew)))
        .unwrap();

    ok(format!("{name} joined the crew!"))
}

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
pub struct OpenedChest;

fn open_chest(world: &mut World, performer: Entity, chest: Entity) -> Result {
    let chest_pos = *world.get::<&Pos>(chest).unwrap();

    position::move_adjacent(world, performer, chest_pos)?;

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

pub struct Success {
    message: Option<String>,
    areas: Option<Vec<Entity>>,
}

fn ok(message: String) -> Result {
    Ok(Success {
        message: Some(message),
        areas: None,
    })
}

fn ok_at(message: String, areas: Vec<Entity>) -> Result {
    Ok(Success {
        message: Some(message),
        areas: Some(areas),
    })
}

fn silent_ok() -> Result {
    Ok(Success {
        message: None,
        areas: None,
    })
}
