use crate::action::{self, Action};
use crate::asset::{CrewData, GameAssets};
use crate::core::area::{self, FuelAmount, ShipState, ShipStatus};
use crate::core::behavior::{
    self, Character, CrewLossMemory, Hostile, RepeatingAction, TalkedAboutEnoughFuel, Waiting,
};
use crate::core::inventory::Held;
use crate::core::item::ItemType;
use crate::core::name::{self, ArticleKind, Name, NameData, NameIdData, NameQuery};
use crate::core::position::{self, Pos};
use crate::core::status::{self, Health, Morale, Stamina, Trait};
use crate::core::{CrewMember, OpenedChest};
use crate::game_interface::{Phase, PhaseResult};
use crate::location::{self, GenerationState, InitialSpawnData, PickResult};
use crate::view::text::{self, CombinableMsgType};
use crate::view::{self, Frame, StatusCache};
use crate::{StopType, ai, command, dialogue, serialization};
use hecs::{CommandBuffer, Entity, Satisfies, World};
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct GameState {
    #[serde(with = "serialization::world")]
    pub world: World,
    #[serde(skip)]
    pub rng: ThreadRng,
    pub generation_state: GenerationState,
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

pub enum Step {
    PrepareNextLocation,
    LoadLocation(String),
    PrepareTick,
    Tick(Option<(Action, command::Target)>),
    ChangeControlled(Entity),
}

pub fn run(
    mut step: Step,
    state: &mut GameState,
    assets: &GameAssets,
) -> (PhaseResult, Vec<Frame>) {
    let mut view_buffer = view::Buffer::new(assets);
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
                    NameData::find(&state.world, state.controlled, view_buffer.assets).definite()
                ));
                state.has_introduced_controlled = true;
            }

            view_buffer.capture_view(state, false);
            dialogue::trigger_landing_dialogue(state, view_buffer);
            Ok(Step::PrepareTick)
        }
        Step::PrepareTick => {
            view_buffer.flush_hint(state);
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
            view_buffer.capture_view(state, false);
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
        && behavior::is_safe(
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
        view_buffer.capture_view(state, true);
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

    dialogue::trigger_encounter_dialogue(state, view_buffer);

    handle_was_waiting(state, view_buffer);

    if is_ship_launching(state) {
        leave_location(state, view_buffer);
        dialogue::trigger_ship_dialogue(state, view_buffer);
        for (_, memory) in state.world.query::<&mut CrewLossMemory>().iter() {
            memory.recent = false;
        }
        Ok(Step::PrepareNextLocation)
    } else {
        let area = state
            .world
            .get::<&Pos>(state.controlled)
            .unwrap()
            .get_area();
        if area != prev_area {
            view_buffer.capture_view(state, false);
        }
        Ok(Step::PrepareTick)
    }
}

fn should_take_user_input(state: &GameState) -> bool {
    !state
        .world
        .satisfies::<hecs::Or<&RepeatingAction, &status::IsStunned>>(state.controlled)
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

    ai::tick(
        &mut action_map,
        &mut state.world,
        &mut state.rng,
        view_buffer.assets,
    );

    action::tick(action_map, state, view_buffer);

    status::detect_low_health(&mut state.world, view_buffer, state.controlled);
    status::detect_low_stamina(&mut state.world, view_buffer, state.controlled);

    handle_crew_deaths(state, view_buffer);
    drop_objects_held_by_the_dead(&mut state.world);

    let mut buffer = CommandBuffer::new();

    for &stun_recovering_entity in &stun_recovering_entities {
        buffer.remove_one::<status::IsStunned>(stun_recovering_entity);
    }
    let alive_recovering_entities = stun_recovering_entities
        .into_iter()
        .filter(|&entity| status::is_alive(entity, &state.world))
        .map(|entity| NameIdData::find(&state.world, entity))
        .collect::<Vec<_>>();
    if !alive_recovering_entities.is_empty() {
        view_buffer.messages.add(format!(
            "{entities} regained their balance.",
            entities = text::join_elements(name::names_with_counts(
                alive_recovering_entities,
                name::ArticleKind::The,
                name::CountFormat::Text,
                view_buffer.assets,
            ))
        ))
    }

    for (item, (_, held)) in state
        .world
        .query::<(&ItemType, &Held)>()
        .into_iter()
        .filter(|&(_, (item_type, _))| *item_type == ItemType::FourLeafClover)
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
            NameData::find_by_ref(holder_ref, view_buffer.assets).definite(),
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
            NameData::find(&state.world, aftik, view_buffer.assets).definite(),
        ));
    }

    if !status::is_alive(state.controlled, &state.world) {
        state.status_cache = StatusCache::default();
        view_buffer.capture_view(state, false);
    }

    let mut buffer = CommandBuffer::new();
    for character in dead_crew {
        state.world.remove_one::<CrewMember>(character).unwrap();
        for (_, morale) in state.world.query_mut::<&mut Morale>().with::<&CrewMember>() {
            morale.crew_death_effect();
        }
        if let Ok(Name { name, .. }) = state.world.get::<&Name>(character).as_deref() {
            for (crew_member, ()) in state.world.query::<()>().with::<&CrewMember>().iter() {
                buffer.insert_one(
                    crew_member,
                    CrewLossMemory {
                        name: name.clone(),
                        recent: true,
                    },
                );
            }
        }
    }
    buffer.run_on(&mut state.world);
}

fn drop_objects_held_by_the_dead(world: &mut World) {
    let mut buffer = CommandBuffer::new();
    for (entity, held) in world.query::<&Held>().iter() {
        let Ok(holder_ref) = world.entity(held.holder) else {
            buffer.despawn(entity);
            continue;
        };
        if holder_ref
            .get::<&Health>()
            .is_some_and(|health| health.is_dead())
        {
            let Some(pos) = holder_ref.get::<&Pos>() else {
                buffer.despawn(entity);
                continue;
            };

            buffer.remove_one::<Held>(entity);
            buffer.insert_one(entity, *pos);
        }
    }
    buffer.run_on(world);
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
        view_buffer.capture_view(state, false);
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
                .get::<&Hostile>(entity)
                .is_ok_and(|hostile| !hostile.aggressive)
            {
                position::turn_towards(&state.world, entity, player_pos);
                view_buffer.messages.add(
                    CombinableMsgType::Threatening.message(NameIdData::find(&state.world, entity)),
                );
            }

            if state
                .world
                .get::<&Hostile>(entity)
                .is_ok_and(|hostile| hostile.aggressive)
            {
                position::turn_towards(&state.world, entity, player_pos);
                view_buffer.messages.add(
                    CombinableMsgType::Attacking.message(NameIdData::find(&state.world, entity)),
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
        let name = NameData::from_query(query, view_buffer.assets).definite();
        view_buffer.messages.add(format!("{name} was left behind."));
    }
    for (_, morale) in state.world.query_mut::<&mut Morale>().with::<&CrewMember>() {
        morale.dampen(0.6);
    }

    let rations_before_eating = state
        .world
        .query::<(&ItemType, &Pos)>()
        .iter()
        .filter(|&(_, (item_type, pos))| {
            *item_type == ItemType::FoodRation && area::is_in_ship(*pos, &state.world)
        })
        .count();
    consume_rations_healing(state, view_buffer);

    view_buffer.capture_view(state, false);

    location::despawn_all_except_ship(&mut state.world);
    state
        .world
        .get::<&mut ShipState>(state.ship_core)
        .unwrap()
        .status = ShipStatus::NeedFuel(FuelAmount::TwoCans);

    let crew = state.world.get::<&CrewMember>(state.controlled).unwrap().0;
    let _ = state.world.remove_one::<TalkedAboutEnoughFuel>(crew);

    status::apply_morale_effects_from_crew_state(&mut state.world, rations_before_eating);
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
        .query::<(&ItemType, &Held)>()
        .iter()
        .filter(|&(_, (item_type, held))| {
            *item_type == ItemType::FoodRation
                && crew_in_ship.iter().any(|&entity| held.held_by(entity))
        })
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

fn consume_rations_healing(state: &mut GameState, view_buffer: &mut view::Buffer) {
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
            .query::<(&ItemType, &Pos)>()
            .iter()
            .filter(|&(_, (item_type, pos))| {
                *item_type == ItemType::FoodRation && area::is_in_ship(*pos, &state.world)
            })
            .take(usize::from(rations_to_eat))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        if !rations.is_empty() {
            let rations_factor = f32::from(rations.len() as u16) / f32::from(rations_to_eat);
            let heal_fraction = rations_factor * status::get_food_heal_fraction(entity_ref);
            entity_ref
                .get::<&mut Health>()
                .unwrap()
                .restore_fraction(heal_fraction, entity_ref);
            crew_eating_rations.push((crew_candidate, rations.len() as u16));

            for ration in rations {
                state.world.despawn(ration).unwrap();
            }
        }
    }
    if !crew_eating_rations.is_empty() {
        view_buffer.messages.add(build_eating_message(
            crew_eating_rations,
            &state.world,
            view_buffer.assets,
        ));
    }
}

fn build_eating_message(
    crew_eating_rations: Vec<(Entity, u16)>,
    world: &World,
    assets: &GameAssets,
) -> String {
    if let &[(entity, amount)] = &crew_eating_rations[..] {
        format!(
            "{the_character} ate {one_ration} to recover some health.",
            the_character = NameData::find(world, entity, assets).definite(),
            one_ration = assets
                .noun_data_map
                .lookup(&ItemType::FoodRation.noun_id())
                .with_text_count(amount, ArticleKind::One),
        )
    } else {
        let names = crew_eating_rations
            .iter()
            .map(|(entity, _)| NameData::find(world, *entity, assets).definite())
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
    state.controlled = character;

    view_buffer.messages.add(format!(
        "You're now playing as the aftik {}.",
        NameData::find(&state.world, character, view_buffer.assets).definite()
    ));
}
