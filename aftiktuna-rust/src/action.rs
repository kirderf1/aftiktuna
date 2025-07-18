use crate::core::item::{FoodRation, Type as ItemType};
use crate::core::name::{Name, NameData};
use crate::core::position::{self, Direction, Pos};
use crate::core::{
    self, CrewMember, FortunaChest, Hostile, OpenedChest, Recruitable, RepeatingAction, inventory,
    status,
};
use crate::game_loop::GameState;
use crate::view::text::{IntoMessage, Message};
use crate::view::{self, Frame};
use hecs::{Entity, World};
use std::collections::HashMap;
use std::result;

mod combat;
mod dialogue;
mod door;
pub mod item;
mod ship;
pub mod trade;

#[derive(Debug, Clone)]
pub enum Action {
    TakeItem(Entity, NameData),
    TakeAll,
    Search(item::SearchAction),
    GiveItem(Entity, Entity),
    Wield(Entity, NameData),
    Use(item::UseAction),
    EnterDoor(Entity),
    ForceDoor(Entity, bool),
    GoToShip,
    Attack(Vec<Entity>),
    Wait,
    Examine(Entity),
    Rest(bool),
    Refuel,
    Launch,
    TalkTo(Entity),
    Recruit(Entity),
    TellToWait(Entity),
    TellToWaitAtShip(Entity),
    TellToFollow(Entity),
    Trade(Entity),
    Buy(ItemType, u16),
    Sell(Vec<Entity>),
    ExitTrade,
    OpenChest(Entity),
    Tame(Entity),
    Name(Entity, String),
}

pub fn tick(
    action_map: HashMap<Entity, Action>,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    let mut entities = action_map
        .into_iter()
        .map(|(entity, action)| {
            (
                (entity, action),
                state
                    .world
                    .get::<&status::Stats>(entity)
                    .map_or(0, |stats| stats.agility),
            )
        })
        .collect::<Vec<_>>();

    entities.sort_by(|(_, agility1), (_, agility2)| agility2.cmp(agility1));

    for ((entity, action), _) in entities {
        if can_act(entity, &state.world) {
            perform(state, entity, action, view_buffer);
        }
    }
}

fn can_act(entity: Entity, world: &World) -> bool {
    match world.entity(entity) {
        Ok(entity_ref) => {
            status::is_alive_ref(entity_ref) && !entity_ref.satisfies::<&status::IsStunned>()
        }
        Err(_) => false,
    }
}

fn perform(
    state: &mut GameState,
    performer: Entity,
    action: Action,
    view_buffer: &mut view::Buffer,
) {
    let context = Context {
        state,
        dialogue_context: DialogueContext { view_buffer },
    };
    use Action::*;
    let result = match action {
        OpenChest(chest) => open_chest(&mut state.world, performer, chest),
        TakeItem(item, name) => item::take_item(&mut state.world, performer, item, name),
        Search(search_action) => search_action.run(performer, context),
        TakeAll => item::take_all(&mut state.world, performer),
        GiveItem(item, receiver) => item::give_item(context, performer, item, receiver),
        Wield(item, name) => item::wield(&mut state.world, performer, item, name),
        Use(use_action) => use_action.run(performer, context),
        EnterDoor(door) => door::enter_door(state, performer, door),
        ForceDoor(door, assisting) => door::force_door(context, performer, door, assisting),
        GoToShip => door::go_to_ship(context, performer),
        Attack(targets) => combat::attack(state, performer, targets),
        Wait => {
            state.world.insert_one(performer, WasWaiting).unwrap();
            silent_ok()
        }
        Examine(target) => {
            if let Some(target_pos) = state.world.get::<&Pos>(target).ok().map(|pos| *pos) {
                let _ = position::move_adjacent(&mut state.world, performer, target_pos);
                let performer_pos = *state.world.get::<&Pos>(performer).unwrap();
                if performer_pos != target_pos {
                    state
                        .world
                        .insert_one(performer, Direction::between(performer_pos, target_pos))
                        .unwrap();
                }
            }
            silent_ok()
        }
        Rest(first) => rest(&mut state.world, performer, first),
        Refuel => ship::refuel(state, performer),
        Launch => ship::launch(state, performer),
        TalkTo(target) => dialogue::talk_to(context, performer, target),
        Recruit(target) => dialogue::recruit(context, performer, target),
        TellToWait(target) => dialogue::tell_to_wait(context, performer, target),
        TellToWaitAtShip(target) => dialogue::tell_to_wait_at_ship(context, performer, target),
        TellToFollow(target) => dialogue::tell_to_follow(context, performer, target),
        Trade(shopkeeper) => trade::trade(&mut state.world, performer, shopkeeper),
        Buy(item_type, amount) => trade::buy(&mut state.world, performer, item_type, amount),
        Sell(items) => trade::sell(&mut state.world, performer, items),
        ExitTrade => trade::exit(&mut state.world, performer),
        Tame(target) => tame(&mut state.world, performer, target),
        Name(target, name) => give_name(&mut state.world, performer, target, name),
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
        Err(error) => {
            if performer == controlled
                || error.visible
                    && world.get::<&Pos>(performer).is_ok_and(|pos| {
                        pos.is_in(world.get::<&Pos>(controlled).unwrap().get_area())
                    })
            {
                view_buffer.messages.add(error.message);
                view_buffer.capture_view(state);
            }
        }
    }
}

pub struct WasWaiting;

struct Context<'a> {
    state: &'a mut GameState,
    dialogue_context: DialogueContext<'a>,
}

impl<'a> Context<'a> {
    fn mut_world(&mut self) -> &mut World {
        &mut self.state.world
    }
}

struct DialogueContext<'a> {
    view_buffer: &'a mut view::Buffer,
}

impl<'a> DialogueContext<'a> {
    fn capture_frame_for_dialogue(&mut self, state: &mut GameState) {
        if !self.view_buffer.messages.is_empty() {
            self.view_buffer.capture_view(state);
        }
    }

    fn add_dialogue(&mut self, world: &World, target: Entity, message: impl Into<String>) {
        self.view_buffer
            .push_frame(Frame::new_dialogue(world, target, vec![message.into()]));
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
        world.insert_one(performer, RepeatingAction::Rest).unwrap();
    }

    if first_turn_resting {
        ok("The crew takes some time to rest up.".to_string())
    } else {
        silent_ok()
    }
}

fn open_chest(world: &mut World, performer: Entity, chest: Entity) -> Result {
    let chest_pos = *world.get::<&Pos>(chest).unwrap();

    position::move_adjacent(world, performer, chest_pos)?;

    if world.get::<&FortunaChest>(chest).is_err() {
        return Err(Error::visible(format!(
            "{} tried to open {}, but that is not the fortuna chest!",
            NameData::find(world, performer).definite(),
            NameData::find(world, chest).definite()
        )));
    }

    world.insert_one(performer, OpenedChest).unwrap();
    ok(format!(
        "{} opened the fortuna chest and found the item that they desired the most.",
        NameData::find(world, performer).definite()
    ))
}

fn tame(world: &mut World, performer: Entity, target: Entity) -> Result {
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let crew_size = world.query::<&CrewMember>().iter().count();
    if crew_size >= core::CREW_SIZE_LIMIT {
        return Err(Error::private(
            "There is not enough room for another crew member.",
        ));
    }

    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    let target_pos = *world.get::<&Pos>(target).unwrap();

    if !status::is_alive(target, world) {
        return Err(Error::private(format!(
            "{target_name} is not a tameable creature."
        )));
    }

    {
        let mut query = world
            .query_one::<&Hostile>(target)
            .unwrap()
            .with::<&Recruitable>();
        let Some(hostile) = query.get() else {
            return Err(Error::private(format!(
                "{target_name} is not a tameable creature."
            )));
        };
        if hostile.aggressive {
            return Err(Error::private(format!(
                "{target_name} is on the attack and does not let {performer_name} approach it."
            )));
        }
    }

    let creature_count = world
        .query::<&Pos>()
        .with::<&Hostile>()
        .iter()
        .filter(|(_, pos)| pos.is_in(target_pos.get_area()))
        .count();
    if creature_count > 1 {
        return Err(Error::private(format!(
            "{performer_name} is unable to approach {target_name} as the latter is not alone."
        )));
    }

    position::move_adjacent(world, performer, target_pos)?;

    inventory::consume_one::<&FoodRation>(world, performer).ok_or_else(|| {
        Error::private(format!("{performer_name} needs a food ration for taming."))
    })?;

    world
        .exchange_one::<Hostile, _>(target, CrewMember(crew))
        .unwrap();
    ok(format!(
        "{performer_name} offered a food ration to {target_name} and tamed it."
    ))
}

fn give_name(world: &mut World, performer: Entity, target: Entity, name: String) -> Result {
    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    let target_pos = *world.get::<&Pos>(target).unwrap();

    {
        let target = world.entity(target).unwrap();
        if !target.has::<CrewMember>() {
            return Err(Error::private(format!(
                "{performer_name} cannot name {target_name}."
            )));
        }

        if target.has::<Name>() {
            return Err(Error::private(format!("{target_name} already has a name.")));
        }
    }

    position::move_adjacent(world, performer, target_pos)?;

    world.insert_one(target, Name::known(&name)).unwrap();
    ok(format!(
        "{performer_name} dubbed {target_name} to be named {name}."
    ))
}

type Result = result::Result<Success, Error>;

pub struct Success {
    message: Option<Message>,
    areas: Option<Vec<Entity>>,
}

fn ok(message: impl IntoMessage) -> Result {
    Ok(Success {
        message: Some(message.into_message()),
        areas: None,
    })
}

fn ok_at(message: impl IntoMessage, areas: Vec<Entity>) -> Result {
    Ok(Success {
        message: Some(message.into_message()),
        areas: Some(areas),
    })
}

fn silent_ok() -> Result {
    Ok(Success {
        message: None,
        areas: None,
    })
}

pub struct Error {
    message: String,
    visible: bool,
}

impl Error {
    fn private(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            visible: false,
        }
    }

    fn visible(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            visible: true,
        }
    }
}

impl<T: Into<String>> From<T> for Error {
    fn from(value: T) -> Self {
        Self::private(value)
    }
}
