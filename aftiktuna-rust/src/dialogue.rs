use crate::core::behavior::{
    Character, CrewLossMemory, EncounterDialogue, GivesHuntReward, Recruitable, Talk, Waiting,
};
use crate::core::display::DialogueExpression;
use crate::core::name::Name;
use crate::core::position::{self, Direction, Pos};
use crate::core::status::Health;
use crate::core::{CrewMember, Tag};
use crate::game_loop::GameState;
use crate::view;
use hecs::{Entity, World};
use rand::seq::{IteratorRandom, SliceRandom};

pub fn talk_dialogue(
    performer: Entity,
    target: Entity,
    world: &mut World,
    view_buffer: &mut view::Buffer,
) {
    let target_ref = world.entity(target).unwrap();
    if target_ref.get::<&Name>().is_some_and(|name| !name.is_known) {
        let name_string = {
            let mut name_ref = target_ref.get::<&mut Name>().unwrap();
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
    } else {
        regular_greeting(performer, target, world, view_buffer);
    }

    let gives_hunt_reward = target_ref.get::<&GivesHuntReward>();
    if gives_hunt_reward.is_some() {
        let gives_hunt_reward = gives_hunt_reward.unwrap();
        if any_alive_with_tag(&gives_hunt_reward.target_tag, world) {
            view_buffer.push_dialogue(
                world,
                target,
                gives_hunt_reward.task_dialogue.expression,
                &gives_hunt_reward.task_dialogue.message,
            );
        } else {
            drop(gives_hunt_reward);
            let GivesHuntReward {
                reward_dialogue,
                reward,
                ..
            } = world.remove_one::<GivesHuntReward>(target).unwrap();

            view_buffer.push_dialogue(
                world,
                target,
                reward_dialogue.expression,
                reward_dialogue.message,
            );

            reward.give_reward_to(performer, world);
        }
    } else if let Some(talk) = target_ref.get::<&Talk>() {
        view_buffer.push_dialogue(world, target, talk.0.expression, &talk.0.message);
    } else if target_ref.has::<Recruitable>() {
        view_buffer.push_dialogue(
            world,
            target,
            DialogueExpression::Neutral,
            "I wish I could leave this place and go on an adventure.",
        );
    }
}

fn regular_greeting(
    performer: Entity,
    target: Entity,
    world: &World,
    view_buffer: &mut view::Buffer,
) {
    let target_ref = world.entity(target).unwrap();

    view_buffer.push_dialogue(world, performer, DialogueExpression::Excited, "Hi!");

    if target_ref
        .get::<&GivesHuntReward>()
        .is_some_and(|gives_hunt_reward| any_alive_with_tag(&gives_hunt_reward.target_tag, world))
    {
        view_buffer.push_dialogue(
            world,
            target,
            DialogueExpression::Neutral,
            "Hello! I have a bit of a problem at the moment.",
        );
    } else if target_ref.has::<Waiting>() {
        view_buffer.push_dialogue(
            world,
            target,
            DialogueExpression::Neutral,
            "Hello! I'll continue to wait here until you tell me otherwise.",
        );
    } else if target_ref
        .get::<&Health>()
        .is_some_and(|health| health.is_badly_hurt())
    {
        view_buffer.push_dialogue(world, target, DialogueExpression::Neutral, "Hello! I'm not doing too well right now. Perhaps I should stay behind if we will be exploring anything more.");
    } else {
        view_buffer.push_dialogue(world, target, DialogueExpression::Excited, "Hello!");
    }
}

fn any_alive_with_tag(target_tag: &Tag, world: &World) -> bool {
    world
        .query::<(&Health, &Tag)>()
        .iter()
        .any(|(_, (health, tag))| health.is_alive() && target_tag == tag)
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
        state
            .world
            .insert_one(character1, Direction::Right)
            .unwrap();
        state.world.insert_one(character2, Direction::Left).unwrap();
        ship_dialogue(character1, character2, state, view_buffer);
    }
}

fn ship_dialogue(
    character1: Entity,
    character2: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    let badly_hurt1 = state
        .world
        .get::<&Health>(character1)
        .is_ok_and(|health| health.is_badly_hurt());
    let badly_hurt2 = state
        .world
        .get::<&Health>(character2)
        .is_ok_and(|health| health.is_badly_hurt());
    if state.generation_state.locations_before_fortuna() == 0 {
        if badly_hurt1 {
            view_buffer.push_dialogue(
                &state.world,
                character1,
                DialogueExpression::Neutral,
                "Looks like we are arriving at the Fortuna crash site next. Do you think that we will make it?",
            );
            view_buffer.push_dialogue(
                &state.world,
                character2,
                DialogueExpression::Neutral,
                "I hope so.",
            );
        } else {
            view_buffer.push_dialogue(
                &state.world,
                character1,
                DialogueExpression::Excited,
                "Looks like we are arriving at the Fortuna crash site next. Are you excited?",
            );
            if let Ok(memory) = state.world.get::<&CrewLossMemory>(character2) {
                view_buffer.push_dialogue(
                    &state.world,
                    character2,
                    DialogueExpression::Sad,
                    format!(
                        "Yeah. I just wish {name} was with us too.",
                        name = memory.name,
                    ),
                );
            } else if badly_hurt2 {
                view_buffer.push_dialogue(
                    &state.world,
                    character2,
                    DialogueExpression::Neutral,
                    "Yeah, but I am also worried.",
                );
            } else {
                view_buffer.push_dialogue(
                    &state.world,
                    character2,
                    DialogueExpression::Excited,
                    "Yeah, I think so!",
                );
            }
        }
    } else if let Ok(memory) = state.world.get::<&CrewLossMemory>(character1)
        && memory.recent
    {
        view_buffer.push_dialogue(
            &state.world,
            character1,
            DialogueExpression::Sad,
            format!(
                "I am sad that we lost {name}. Do you think that we will make it?",
                name = memory.name
            ),
        );
        if badly_hurt2 {
            view_buffer.push_dialogue(
                &state.world,
                character2,
                DialogueExpression::Neutral,
                "I am not sure, but I hope so.",
            );
        } else {
            view_buffer.push_dialogue(
                &state.world,
                character2,
                DialogueExpression::Neutral,
                "Don't worry. I'm sure we will.",
            );
        }
    } else if badly_hurt1 {
        view_buffer.push_dialogue(
            &state.world,
            character1,
            DialogueExpression::Neutral,
            "Will we be able to go somewhere safer next?",
        );
        view_buffer.push_dialogue(
            &state.world,
            character2,
            DialogueExpression::Neutral,
            "I don't know. Let's see what our options are.",
        );
    } else if !badly_hurt1 && badly_hurt2 {
        view_buffer.push_dialogue(
            &state.world,
            character1,
            DialogueExpression::Neutral,
            "That worked out in the end, right?",
        );
        view_buffer.push_dialogue(
            &state.world,
            character2,
            DialogueExpression::Neutral,
            "I guess so. But can we go somewhere safer next?",
        );
        view_buffer.push_dialogue(
            &state.world,
            character1,
            DialogueExpression::Neutral,
            "I don't know. Let's see what our options are.",
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
}
