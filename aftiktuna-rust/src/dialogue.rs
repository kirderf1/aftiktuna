use crate::asset::dialogue::ConditionedDialogueNode;
use crate::core::behavior::{
    self, BackgroundDialogue, Character, CrewLossMemory, EncounterDialogue, GivesHuntReward,
    Recruitable, Talk, TalkState, TalkedAboutEnoughFuel,
};
use crate::core::display::DialogueExpression;
use crate::core::name::Name;
use crate::core::position::{self, Pos};
use crate::core::status::Health;
use crate::core::{self, CrewMember, area, inventory};
use crate::game_loop::GameState;
use crate::{asset, view};
use hecs::{Entity, World};
use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom};

/// Expects dialogue setup (placement and frame capture) to already be done.
pub fn ask_name_dialogue(
    performer: Entity,
    target: Entity,
    world: &mut World,
    view_buffer: &mut view::Buffer,
) {
    let name_string = {
        let mut name_ref = world.get::<&mut Name>(target).unwrap();
        name_ref.is_known = true;
        name_ref.name.clone()
    };
    view_buffer.push_dialogue(
        world,
        performer,
        DialogueExpression::Neutral,
        "Hi! What is your name?",
    );
    view_buffer.push_dialogue(
        world,
        target,
        DialogueExpression::Neutral,
        format!("My name is {name_string}."),
    );
}

/// Expects dialogue setup (placement and frame capture) to already be done.
pub fn prompt_npc_dialogue(
    crew_member: Entity,
    npc: Entity,
    world: &mut World,
    view_buffer: &mut view::Buffer,
) {
    let npc_ref = world.entity(npc).unwrap();
    let gives_hunt_reward = npc_ref.get::<&GivesHuntReward>();
    if gives_hunt_reward.is_some() {
        let gives_hunt_reward = gives_hunt_reward.unwrap();
        if !gives_hunt_reward.is_fulfilled(world) {
            view_buffer.push_dialogue(
                world,
                npc,
                gives_hunt_reward.task_dialogue.expression,
                &gives_hunt_reward.task_dialogue.message,
            );
        } else {
            drop(gives_hunt_reward);
            complete_hunt_quest(crew_member, npc, world, view_buffer);
        }
    } else if let Some(talk) = npc_ref.get::<&Talk>() {
        view_buffer.push_dialogue(world, npc, talk.0.expression, &talk.0.message);
    } else if npc_ref.has::<Recruitable>() {
        view_buffer.push_dialogue(
            world,
            npc,
            DialogueExpression::Neutral,
            "I wish I could leave this place and go on an adventure.",
        );
    }
}

/// Expects dialogue setup (placement and frame capture) to already be done.
pub fn complete_hunt_quest(
    crew_member: Entity,
    npc: Entity,
    world: &mut World,
    view_buffer: &mut view::Buffer,
) {
    let GivesHuntReward {
        reward_dialogue,
        reward,
        ..
    } = world.remove_one::<GivesHuntReward>(npc).unwrap();

    view_buffer.push_dialogue(
        world,
        npc,
        reward_dialogue.expression,
        reward_dialogue.message,
    );

    reward.give_reward_to(crew_member, world);
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
        if state.generation_state.locations_before_fortuna() == 0 {
            trigger_dialogue_by_name(
                "on_ship/approaching_fortuna",
                character1,
                character2,
                state,
                view_buffer,
            );
        } else {
            trigger_dialogue_by_name(
                "on_ship/approaching_location",
                character1,
                character2,
                state,
                view_buffer,
            );
        }
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

            position::turn_towards(&mut state.world, speaker, player_pos);
            let EncounterDialogue(dialogue_node) = state
                .world
                .remove_one::<EncounterDialogue>(speaker)
                .unwrap();
            view_buffer.push_dialogue(
                &state.world,
                speaker,
                dialogue_node.expression,
                dialogue_node.message,
            );
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
            let speaker = *possible_speakers.choose(&mut state.rng).unwrap();
            state.world.insert_one(crew, TalkedAboutEnoughFuel).unwrap();

            trigger_dialogue_by_name(
                "obtained_enough_fuel",
                speaker,
                state.controlled,
                state,
                view_buffer,
            );
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
        view_buffer.capture_view_before_dialogue(state);

        position::turn_towards(&mut state.world, speaker, target_pos);
        position::turn_towards(&mut state.world, target, speaker_pos);
        let speakers = [speaker, target];
        for i in 0..background_dialogue.dialogue.len() {
            let dialogue_node = &background_dialogue.dialogue[i];
            view_buffer.push_dialogue(
                &state.world,
                speakers[i % 2],
                dialogue_node.expression,
                &dialogue_node.message,
            );
        }
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
    }
}

fn trigger_dialogue_by_name(
    name: &str,
    speaker: Entity,
    target: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    match asset::dialogue::load_dialogue_data(name) {
        Ok(dialogue) => {
            if let Some(dialogue_node) = dialogue.select_node(speaker, target, &state.world) {
                view_buffer.capture_view_before_dialogue(state);
                run_dialogue_node(
                    dialogue_node,
                    speaker,
                    target,
                    &mut state.world,
                    view_buffer,
                );
            }
        }
        Err(error) => println!("Failed to load dialogue {name}: {error}"),
    }
}

fn run_dialogue_node(
    dialogue_node: &ConditionedDialogueNode,
    speaker: Entity,
    target: Entity,
    world: &mut World,
    view_buffer: &mut view::Buffer,
) {
    let target_pos = *world.get::<&Pos>(target).unwrap();
    position::turn_towards(world, speaker, target_pos);

    if dialogue_node.message.contains("{crew_loss_memory_name}")
        && let Ok(crew_loss_memory) = world.get::<&CrewLossMemory>(speaker)
    {
        view_buffer.push_dialogue(
            world,
            speaker,
            dialogue_node.expression,
            dialogue_node
                .message
                .replace("{crew_loss_memory_name}", &crew_loss_memory.name),
        );
    } else {
        view_buffer.push_dialogue(
            world,
            speaker,
            dialogue_node.expression,
            &dialogue_node.message,
        );
    }

    if let Some(reply) = &dialogue_node.reply
        && let Some(reply_node) = reply.select_node(target, speaker, world)
    {
        run_dialogue_node(reply_node, target, speaker, world, view_buffer);
    }
}
