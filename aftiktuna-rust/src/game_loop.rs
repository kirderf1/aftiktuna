use std::collections::HashMap;

use hecs::{CommandBuffer, Entity, Satisfies, World};
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

use crate::action::{self, Action};
use crate::asset::CrewData;
use crate::core::area::{self, FuelAmount, ShipState, ShipStatus};
use crate::core::display::OrderWeight;
use crate::core::inventory::Held;
use crate::core::item::{self, FoodRation, FourLeafClover};
use crate::core::name::{Article, NameData, NameQuery};
use crate::core::position::{Direction, Pos};
use crate::core::status::{Health, Stamina, Trait};
use crate::core::{
    self, Character, CrewMember, OpenedChest, RepeatingAction, Waiting, inventory, status,
};
use crate::game_interface::{Phase, PhaseResult};
use crate::location::{self, GenerationState, InitialSpawnData, PickResult};
use crate::view::text::{self, CombinableMsgType, Messages};
use crate::view::{self, Frame, StatusCache};
use crate::{StopType, ai, command, serialization};

#[derive(Serialize, Deserialize)]
pub struct GameState {
    #[serde(with = "serialization::world")]
    pub world: World,
    #[serde(skip)]
    pub rng: ThreadRng,
    pub generation_state: GenerationState,
    #[serde(alias = "ship")] // backwards-compatibility with 4.0
    pub ship_core: Entity,
    pub controlled: Entity,
    pub status_cache: StatusCache,
    pub has_introduced_controlled: bool,
}

pub fn setup(mut generation_state: GenerationState) -> GameState {
    let InitialSpawnData {
        world,
        controlled_character,
        ship_core,
    } = location::spawn_starting_crew_and_ship(
        CrewData::load_starting_crew().expect("Unable to set up game"),
        &mut generation_state,
    )
    .unwrap();

    GameState {
        world,
        rng: rand::rng(),
        generation_state,
        ship_core,
        controlled: controlled_character,
        status_cache: StatusCache::default(),
        has_introduced_controlled: false,
    }
}

#[derive(Debug)]
pub enum Step {
    PrepareNextLocation,
    LoadLocation(String),
    PrepareTick,
    Tick(Option<(Action, command::Target)>),
    ChangeControlled(Entity),
}

pub fn run(mut step: Step, state: &mut GameState) -> (PhaseResult, Vec<Frame>) {
    let mut view_buffer = view::Buffer::default();
    let phase_result = loop {
        let result = run_step(step, state, &mut view_buffer);
        match result {
            Ok(next_step) => step = next_step,
            Err(phase_result) => break phase_result,
        }
    };

    #[cfg(feature = "debug_logging")]
    crate::serialization::check_world_components(&state.world);

    (phase_result, view_buffer.into_frames())
}

fn run_step(
    phase: Step,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<Step, PhaseResult> {
    match phase {
        Step::PrepareNextLocation => Ok(prepare_next_location(state, view_buffer)?),
        Step::LoadLocation(location) => {
            location::setup_location_into_game(&location, &mut view_buffer.messages, state)
                .map_err(|message| Phase::LoadLocation(location).with_error(message))?;
            if !state.has_introduced_controlled {
                view_buffer.messages.add(format!(
                    "You're playing as the aftik {}.",
                    NameData::find(&state.world, state.controlled).definite()
                ));
                state.has_introduced_controlled = true;
            }

            view_buffer.capture_view(state);
            Ok(Step::PrepareTick)
        }
        Step::PrepareTick => {
            ai::prepare_intentions(&mut state.world);
            Ok(Step::Tick(if should_controlled_character_wait(state) {
                Some((Action::Wait, command::Target::Controlled))
            } else {
                None
            }))
        }
        Step::Tick(chosen_action) => Ok(tick_and_check(chosen_action, state, view_buffer)?),
        Step::ChangeControlled(character) => {
            change_character(state, character, view_buffer);
            view_buffer.capture_view(state);
            Ok(Step::Tick(None))
        }
    }
}

fn prepare_next_location(
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<Step, Phase> {
    match state.generation_state.next(&mut state.rng) {
        PickResult::None => {
            view_buffer.push_ending_frame(&state.world, state.controlled, StopType::Win);
            Err(Phase::Stopped(StopType::Win))
        }
        PickResult::Location(location) => Ok(Step::LoadLocation(location)),
        PickResult::Choice(choice) => {
            view_buffer.push_frame(Frame::LocationChoice(choice.clone()));
            Err(Phase::ChooseLocation(choice))
        }
    }
}

fn should_controlled_character_wait(state: &GameState) -> bool {
    !state
        .world
        .satisfies::<&RepeatingAction>(state.controlled)
        .unwrap()
        && is_wait_requested(&state.world, state.controlled)
        && core::is_safe(
            &state.world,
            state
                .world
                .get::<&Pos>(state.controlled)
                .unwrap()
                .get_area(),
        )
}

fn is_wait_requested(world: &World, controlled: Entity) -> bool {
    let area = world.get::<&Pos>(controlled).unwrap().get_area();
    world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(entity, pos)| *entity != controlled && pos.is_in(area))
        .any(|(entity, _)| ai::is_requesting_wait(world, entity))
}

fn tick_and_check(
    chosen_action: Option<(Action, command::Target)>,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<Step, Phase> {
    if chosen_action.is_none() && should_take_user_input(state) {
        view_buffer.capture_view(state);
        return Err(Phase::CommandInput);
    }

    let prev_area = state
        .world
        .get::<&Pos>(state.controlled)
        .unwrap()
        .get_area();
    tick(chosen_action, state, view_buffer);

    if let Err(stop_type) = check_player_state(state, view_buffer) {
        view_buffer.push_ending_frame(&state.world, state.controlled, stop_type);
        return Err(Phase::Stopped(stop_type));
    }

    handle_was_waiting(state, view_buffer);

    if is_ship_launching(state) {
        leave_location(state, view_buffer);
        Ok(Step::PrepareNextLocation)
    } else {
        let area = state
            .world
            .get::<&Pos>(state.controlled)
            .unwrap()
            .get_area();
        if area != prev_area {
            view_buffer.capture_view(state);
        }
        Ok(Step::PrepareTick)
    }
}

fn should_take_user_input(state: &GameState) -> bool {
    !state
        .world
        .satisfies::<&RepeatingAction>(state.controlled)
        .unwrap()
}

fn tick(
    chosen_action: Option<(Action, command::Target)>,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    let stun_recovering_entities = state
        .world
        .query::<()>()
        .with::<&status::IsStunned>()
        .into_iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<Entity>>();

    let mut action_map = HashMap::new();

    if let Some((action, target)) = chosen_action {
        insert_command_action(&mut action_map, action, target, state);
    }

    ai::tick(&mut action_map, &mut state.world, &mut state.rng);

    action::tick(action_map, state, view_buffer);

    status::detect_low_health(
        &mut state.world,
        &mut view_buffer.messages,
        state.controlled,
    );
    status::detect_low_stamina(
        &mut state.world,
        &mut view_buffer.messages,
        state.controlled,
    );

    handle_crew_deaths(state, view_buffer);

    let mut buffer = CommandBuffer::new();
    for stun_recovering_entity in stun_recovering_entities {
        buffer.remove_one::<status::IsStunned>(stun_recovering_entity);
    }
    for (item, held) in state
        .world
        .query::<&Held>()
        .with::<&FourLeafClover>()
        .into_iter()
    {
        let Ok(holder_ref) = state.world.entity(held.holder) else {
            continue;
        };
        let Some(_) = action::item::FOUR_LEAF_CLOVER_EFFECT.try_apply(holder_ref) else {
            continue;
        };
        buffer.despawn(item);
        view_buffer.messages.add(format!(
            "As {} holds the four leaf clover, it disappears in their hand. (Luck has increased by 2 points)",
            NameData::find_by_ref(holder_ref).definite(),
        ));
    }
    buffer.run_on(&mut state.world);

    for (_, stamina) in state.world.query_mut::<&mut Stamina>() {
        stamina.tick();
    }
}

fn insert_command_action(
    action_map: &mut HashMap<Entity, Action>,
    action: Action,
    target: command::Target,
    state: &GameState,
) {
    match target {
        command::Target::Controlled => {
            action_map.insert(state.controlled, action);
        }
        command::Target::Crew => {
            let area = state
                .world
                .get::<&Pos>(state.controlled)
                .unwrap()
                .get_area();
            for (entity, _) in state
                .world
                .query::<(&Pos, Satisfies<&Waiting>)>()
                .with::<&CrewMember>()
                .iter()
                .filter(|&(entity, (pos, is_waiting))| {
                    pos.is_in(area) && (entity == state.controlled || !is_waiting)
                })
            {
                action_map.insert(entity, action.clone());
            }
        }
    }
}

fn handle_crew_deaths(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let dead_crew = state
        .world
        .query::<&Health>()
        .with::<&CrewMember>()
        .iter()
        .filter(|(_, health)| health.is_dead())
        .map(|(aftik, _)| aftik)
        .collect::<Vec<_>>();

    for &aftik in &dead_crew {
        view_buffer.messages.add(format!(
            "{} is dead.",
            NameData::find(&state.world, aftik).definite()
        ));
    }

    if !status::is_alive(state.controlled, &state.world) {
        state.status_cache = StatusCache::default();
        view_buffer.capture_view(state);
    }

    for character in dead_crew {
        inventory::drop_all_items(&mut state.world, character);
        state.world.remove_one::<CrewMember>(character).unwrap();
    }
}

fn check_player_state(
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) -> Result<(), StopType> {
    if state.world.get::<&CrewMember>(state.controlled).is_err() {
        let (next_character, _) = state
            .world
            .query::<()>()
            .with::<(&CrewMember, &Character)>()
            .iter()
            .next()
            .ok_or(StopType::Lose)?;
        change_character(state, next_character, view_buffer);
    }

    if state.world.get::<&OpenedChest>(state.controlled).is_ok() {
        view_buffer.capture_view(state);
        return Err(StopType::Win);
    }

    Ok(())
}

fn handle_was_waiting(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let player_pos = *state.world.get::<&Pos>(state.controlled).unwrap();

    let entities = state
        .world
        .query::<()>()
        .with::<&action::WasWaiting>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in entities {
        let pos = *state.world.get::<&Pos>(entity).unwrap();
        if pos.is_in(player_pos.get_area()) && status::is_alive(entity, &state.world) {
            if state
                .world
                .get::<&core::Hostile>(entity)
                .map_or(false, |hostile| !hostile.aggressive)
            {
                state
                    .world
                    .insert_one(entity, Direction::between(pos, player_pos))
                    .unwrap();
                view_buffer.messages.add(
                    CombinableMsgType::Threatening.message(NameData::find(&state.world, entity)),
                );
            }

            if state
                .world
                .get::<&core::Hostile>(entity)
                .map_or(false, |hostile| hostile.aggressive)
            {
                state
                    .world
                    .insert_one(entity, Direction::between(pos, player_pos))
                    .unwrap();
                view_buffer.messages.add(
                    CombinableMsgType::Attacking.message(NameData::find(&state.world, entity)),
                );
            }
        }
        state
            .world
            .remove_one::<action::WasWaiting>(entity)
            .unwrap();
    }
}

fn is_ship_launching(state: &GameState) -> bool {
    state
        .world
        .get::<&ShipState>(state.ship_core)
        .is_ok_and(|ship_state| ship_state.status == ShipStatus::Launching)
}

fn leave_location(state: &mut GameState, view_buffer: &mut view::Buffer) {
    deposit_items_to_ship(state);

    view_buffer
        .messages
        .add("The ship leaves for the next planet.");

    for (_, (_, query)) in state
        .world
        .query::<(&Pos, NameQuery)>()
        .with::<&CrewMember>()
        .iter()
        .filter(|&(_, (pos, _))| !area::is_in_ship(*pos, &state.world))
    {
        let name = NameData::from(query).definite();
        view_buffer.messages.add(format!("{name} was left behind."));
    }
    consume_rations_healing(state, &mut view_buffer.messages);

    view_buffer.capture_view(state);

    location::despawn_all_except_ship(&mut state.world);
    state
        .world
        .get::<&mut ShipState>(state.ship_core)
        .unwrap()
        .status = ShipStatus::NeedFuel(FuelAmount::TwoCans);
}

fn deposit_items_to_ship(state: &mut GameState) {
    let crew_in_ship = state
        .world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .filter(|&(_, pos)| area::is_in_ship(*pos, &state.world))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let items = state
        .world
        .query::<&Held>()
        .with::<&FoodRation>()
        .iter()
        .filter(|&(_, held)| crew_in_ship.iter().any(|&entity| held.held_by(entity)))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let item_pos = state
        .world
        .get::<&ShipState>(state.ship_core)
        .unwrap()
        .item_pos;
    for item in items {
        state.world.exchange_one::<Held, _>(item, item_pos).unwrap();
    }
}

fn consume_rations_healing(state: &mut GameState, messages: &mut Messages) {
    let mut crew_candidates = state
        .world
        .query::<&Health>()
        .with::<&CrewMember>()
        .iter()
        .filter(|&(_, health)| health.is_hurt())
        .map(|(entity, health)| (entity, health.as_fraction()))
        .collect::<Vec<_>>();
    crew_candidates.sort_by(|(_, a), (_, b)| a.total_cmp(b));
    let mut crew_eating_rations = Vec::new();

    for (crew_candidate, _) in crew_candidates {
        let entity_ref = state.world.entity(crew_candidate).unwrap();
        let rations_to_eat: u16 = if Trait::BigEater.ref_has_trait(entity_ref) {
            2
        } else {
            1
        };
        let rations = state
            .world
            .query::<&Pos>()
            .with::<&FoodRation>()
            .iter()
            .filter(|&(_, pos)| area::is_in_ship(*pos, &state.world))
            .take(usize::from(rations_to_eat))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        if !rations.is_empty() {
            let rations_factor = f32::from(rations.len() as u16) / f32::from(rations_to_eat);
            entity_ref
                .get::<&mut Health>()
                .unwrap()
                .restore_fraction(rations_factor * status::get_food_heal_fraction(entity_ref));
            crew_eating_rations.push((crew_candidate, rations.len() as u16));

            for ration in rations {
                state.world.despawn(ration).unwrap();
            }
        }
    }
    if !crew_eating_rations.is_empty() {
        messages.add(build_eating_message(crew_eating_rations, &state.world));
    }
}

fn build_eating_message(crew_eating_rations: Vec<(Entity, u16)>, world: &World) -> String {
    if let &[(entity, amount)] = &crew_eating_rations[..] {
        format!(
            "{the_character} ate {one_ration} to recover some health.",
            the_character = NameData::find(world, entity).definite(),
            one_ration = item::Type::FoodRation
                .noun_data()
                .with_text_count(amount, Article::One),
        )
    } else {
        let names = crew_eating_rations
            .iter()
            .map(|(entity, _)| NameData::find(world, *entity).definite())
            .collect::<Vec<_>>();
        let amount = crew_eating_rations
            .iter()
            .map(|(_, amount)| amount)
            .sum::<u16>();
        format!(
            "{names} ate {amount} food rations to recover some health.",
            names = text::join_elements(names)
        )
    }
}

fn change_character(state: &mut GameState, character: Entity, view_buffer: &mut view::Buffer) {
    let _ = state
        .world
        .insert_one(state.controlled, OrderWeight::Creature);
    state
        .world
        .insert_one(character, OrderWeight::Controlled)
        .unwrap();
    state.controlled = character;

    view_buffer.messages.add(format!(
        "You're now playing as the aftik {}.",
        NameData::find(&state.world, character).definite()
    ));
}
