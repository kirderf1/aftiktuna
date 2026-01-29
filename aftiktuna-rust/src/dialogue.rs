use crate::asset::dialogue::ConditionedDialogueNode;
use crate::core::behavior::{
    self, BackgroundDialogue, Character, CrewLossMemory, EncounterDialogue, GivesHuntRewardData,
    Recruitable, Talk, TalkState, TalkedAboutEnoughFuel,
};
use crate::core::name::Name;
use crate::core::position::{self, Pos};
use crate::core::status::Health;
use crate::core::store::{self, Shopkeeper};
use crate::core::{self, CrewMember, area, inventory};
use crate::game_loop::GameState;
use crate::{asset, view};
use hecs::{Entity, World};
use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom};

#[derive(Clone, Debug)]
pub enum TalkTopic {
    AskName,
    CompleteHuntQuest,
}

impl TalkTopic {
    pub fn pick(target: Entity, world: &World) -> Option<Self> {
        if world.get::<&Name>(target).is_ok_and(|name| !name.is_known) {
            Some(TalkTopic::AskName)
        } else if world
            .get::<&GivesHuntRewardData>(target)
            .is_ok_and(|gives_hunt_reward| gives_hunt_reward.is_fulfilled(world))
        {
            Some(TalkTopic::CompleteHuntQuest)
        } else {
            None
        }
    }

    /// Expects dialogue setup (placement and frame capture) to already be done.
    pub fn perform(
        self,
        performer: Entity,
        target: Entity,
        state: &mut GameState,
        view_buffer: &mut view::Buffer,
    ) {
        match self {
            TalkTopic::AskName => {
                trigger_dialogue_by_name("ask_name", performer, target, state, view_buffer);
                prompt_npc_dialogue(performer, target, state, view_buffer);
            }
            TalkTopic::CompleteHuntQuest => {
                complete_hunt_quest(performer, target, state, view_buffer)
            }
        }
    }
}

/// Expects dialogue setup (placement and frame capture) to already be done.
fn prompt_npc_dialogue(
    crew_member: Entity,
    npc: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    let npc_ref = state.world.entity(npc).unwrap();
    let gives_hunt_reward = npc_ref.get::<&mut GivesHuntRewardData>();
    if gives_hunt_reward.is_some() {
        let mut gives_hunt_reward = gives_hunt_reward.unwrap();

        if !gives_hunt_reward.is_fulfilled(&state.world) {
            gives_hunt_reward.presented = true;
            let dialogue_id = gives_hunt_reward.task_dialogue.clone();
            drop(gives_hunt_reward);
            trigger_dialogue_by_name(&dialogue_id, npc, crew_member, state, view_buffer);
        } else {
            drop(gives_hunt_reward);
            let GivesHuntRewardData {
                already_completed_dialogue,
                reward,
                ..
            } = state.world.remove_one::<GivesHuntRewardData>(npc).unwrap();

            trigger_dialogue_by_name(
                &already_completed_dialogue,
                npc,
                crew_member,
                state,
                view_buffer,
            );

            reward.give_reward_to(crew_member, &mut state.world);
        }
    } else {
        drop(gives_hunt_reward);
        if let Some(talk) = npc_ref.get::<&Talk>().map(crate::deref_clone) {
            trigger_dialogue_by_name(&talk.0, npc, crew_member, state, view_buffer);
        } else if npc_ref.has::<Recruitable>() {
            trigger_dialogue_by_name("recruitable", npc, crew_member, state, view_buffer);
        } else if npc_ref.has::<Shopkeeper>() {
            store::initiate_trade(crew_member, npc, state, view_buffer);
        }
    }
}

/// Expects dialogue setup (placement and frame capture) to already be done.
fn complete_hunt_quest(
    crew_member: Entity,
    npc: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    let GivesHuntRewardData {
        reward_dialogue,
        reward,
        ..
    } = state.world.remove_one::<GivesHuntRewardData>(npc).unwrap();

    trigger_dialogue_by_name(&reward_dialogue, crew_member, npc, state, view_buffer);

    reward.give_reward_to(crew_member, &mut state.world);
}

#[derive(Debug, Clone, Copy)]
enum ShipDialogue {
    ApproachingFortuna,
    CrewLoss,
    Worry,
    NeutralRetrospective,
}

impl ShipDialogue {
    fn dialogue_id(self) -> &'static str {
        match self {
            ShipDialogue::ApproachingFortuna => "on_ship/approaching_fortuna",
            ShipDialogue::CrewLoss => "on_ship/crew_loss",
            ShipDialogue::Worry => "on_ship/worry",
            ShipDialogue::NeutralRetrospective => "on_ship/neutral_retrospective",
        }
    }
}

pub fn trigger_ship_dialogue(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let mut crew_characters = state
        .world
        .query::<()>()
        .with::<(&CrewMember, &Character)>()
        .iter()
        .choose_multiple(&mut state.rng, 2);
    crew_characters.shuffle(&mut state.rng);
    if let [(character1, ()), (character2, ())] = crew_characters[..] {
        let area = state.world.get::<&Pos>(character1).unwrap().get_area();
        state
            .world
            .insert_one(character1, Pos::new(area, 0, &state.world))
            .unwrap();
        state
            .world
            .insert_one(character2, Pos::new(area, 1, &state.world))
            .unwrap();

        let character1_ref = state.world.entity(character1).unwrap();
        let ship_dialogue = if state.generation_state.locations_before_fortuna() == 0 {
            ShipDialogue::ApproachingFortuna
        } else if let Some(crew_loss_memory) = character1_ref.get::<&CrewLossMemory>()
            && crew_loss_memory.recent
        {
            ShipDialogue::CrewLoss
        } else if character1_ref
            .get::<&Health>()
            .is_some_and(|health| health.is_badly_hurt())
        {
            ShipDialogue::Worry
        } else {
            ShipDialogue::NeutralRetrospective
        };

        trigger_dialogue_by_name(
            ship_dialogue.dialogue_id(),
            character1,
            character2,
            state,
            view_buffer,
        );
    }
}

pub fn trigger_encounter_dialogue(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let Ok(player_pos) = state
        .world
        .get::<&Pos>(state.controlled)
        .map(crate::deref_clone)
    else {
        return;
    };
    let entities_with_encounter_dialogue = state
        .world
        .query::<&Pos>()
        .with::<&EncounterDialogue>()
        .into_iter()
        .map(|(entity, pos)| (entity, *pos))
        .collect::<Vec<_>>();
    for (speaker, speaker_pos) in entities_with_encounter_dialogue {
        if player_pos.is_in(speaker_pos.get_area()) {
            view_buffer.capture_view_before_dialogue(state);

            position::turn_towards(&state.world, speaker, player_pos);
            let EncounterDialogue(dialogue_id) = state
                .world
                .remove_one::<EncounterDialogue>(speaker)
                .unwrap();
            trigger_dialogue_by_name(&dialogue_id, speaker, state.controlled, state, view_buffer);
        }
    }

    let entities_with_background_dialogue = state
        .world
        .query::<&Pos>()
        .with::<&BackgroundDialogue>()
        .into_iter()
        .map(|(entity, pos)| (entity, *pos))
        .collect::<Vec<_>>();
    for (speaker, speaker_pos) in entities_with_background_dialogue {
        if player_pos.is_in(speaker_pos.get_area()) {
            trigger_background_dialogue(
                speaker,
                speaker_pos,
                state
                    .world
                    .remove_one::<BackgroundDialogue>(speaker)
                    .unwrap(),
                state,
                view_buffer,
            );
        }
    }

    if behavior::is_safe(&state.world, player_pos.get_area()) {
        let possible_speakers = state
            .world
            .query_mut::<&Pos>()
            .with::<(&CrewMember, &Character)>()
            .into_iter()
            .filter(|&(entity, pos)| entity != state.controlled && pos.is_in(player_pos.get_area()))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        let crew = state.world.get::<&CrewMember>(state.controlled).unwrap().0;
        if !state
            .world
            .satisfies::<&TalkedAboutEnoughFuel>(crew)
            .unwrap()
            && area::fuel_needed_to_launch(&state.world).is_some_and(|fuel_amount| {
                fuel_amount <= inventory::fuel_cans_held_by_crew(&state.world, &[])
            })
        {
            state.world.insert_one(crew, TalkedAboutEnoughFuel).unwrap();
            if let Some(&speaker) = possible_speakers.choose(&mut state.rng) {
                trigger_dialogue_by_name(
                    "obtained_enough_fuel",
                    speaker,
                    state.controlled,
                    state,
                    view_buffer,
                );
            }
        }

        for speaker in possible_speakers {
            let badly_hurt = state
                .world
                .get::<&Health>(speaker)
                .is_ok_and(|health| health.is_badly_hurt());

            if badly_hurt
                && state
                    .world
                    .get::<&TalkState>(speaker)
                    .is_ok_and(|state| !state.talked_about_badly_hurt)
            {
                state
                    .world
                    .get::<&mut TalkState>(speaker)
                    .unwrap()
                    .talked_about_badly_hurt = true;
                trigger_dialogue_by_name(
                    "badly_hurt_after_battle",
                    speaker,
                    state.controlled,
                    state,
                    view_buffer,
                );
            }
        }
    }
}

fn trigger_background_dialogue(
    speaker: Entity,
    speaker_pos: Pos,
    background_dialogue: BackgroundDialogue,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    let Some(target) = core::find_one_entity_with_tag(&background_dialogue.target, &state.world)
    else {
        return;
    };
    let Ok(target_pos) = state.world.get::<&Pos>(target).as_deref().copied() else {
        return;
    };

    if target_pos.is_in(speaker_pos.get_area())
        && state.world.satisfies::<&Character>(target).unwrap()
    {
        trigger_dialogue_by_name(
            &background_dialogue.dialogue,
            speaker,
            target,
            state,
            view_buffer,
        );
    }
}

pub fn trigger_landing_dialogue(state: &mut GameState, view_buffer: &mut view::Buffer) {
    let player_pos = *state.world.get::<&Pos>(state.controlled).unwrap();
    let Some(speaker) = state
        .world
        .query_mut::<&Pos>()
        .with::<(&CrewMember, &Character)>()
        .into_iter()
        .find(|&(entity, pos)| entity != state.controlled && pos.is_in(player_pos.get_area()))
        .map(|(entity, _)| entity)
    else {
        return;
    };

    if area::fuel_needed_to_launch(&state.world).is_some_and(|fuel_amount| {
        fuel_amount <= inventory::fuel_cans_held_by_crew(&state.world, &[])
    }) {
        let crew = state.world.get::<&CrewMember>(speaker).unwrap().0;
        state.world.insert_one(crew, TalkedAboutEnoughFuel).unwrap();

        trigger_dialogue_by_name(
            "landing_with_fuel",
            speaker,
            state.controlled,
            state,
            view_buffer,
        );
    } else {
        trigger_dialogue_by_name(
            "landing_without_enough_fuel",
            speaker,
            state.controlled,
            state,
            view_buffer,
        );
    }
}

pub fn trigger_dialogue_by_name(
    name: &str,
    speaker: Entity,
    target: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    match asset::dialogue::load_dialogue_data(name) {
        Ok(dialogue) => {
            if let Some(dialogue_node) = dialogue.select_node(speaker, target, state) {
                view_buffer.capture_view_before_dialogue(state);
                run_dialogue_node(dialogue_node, speaker, target, state, view_buffer);
            }
        }
        Err(error) => println!("Failed to load dialogue {name}: {error}"),
    }
}

fn run_dialogue_node(
    dialogue_node: &ConditionedDialogueNode,
    speaker: Entity,
    target: Entity,
    state: &GameState,
    view_buffer: &mut view::Buffer,
) {
    let world = &state.world;
    let target_pos = *world.get::<&Pos>(target).unwrap();
    position::turn_towards(world, speaker, target_pos);

    let message = if dialogue_node.message.contains("{name}")
        && let Ok(mut name) = world.get::<&mut Name>(speaker)
    {
        name.is_known = true;
        dialogue_node.message.replace("{name}", &name.name)
    } else if dialogue_node.message.contains("{crew_loss_memory_name}")
        && let Ok(crew_loss_memory) = world.get::<&CrewLossMemory>(speaker)
    {
        dialogue_node
            .message
            .replace("{crew_loss_memory_name}", &crew_loss_memory.name)
    } else {
        dialogue_node.message.clone()
    };

    view_buffer.push_dialogue(world, speaker, dialogue_node.expression, message);

    if let Some(reply) = &dialogue_node.reply
        && let Some(reply_node) = reply.select_node(target, speaker, state)
    {
        run_dialogue_node(reply_node, target, speaker, state, view_buffer);
    }
}
