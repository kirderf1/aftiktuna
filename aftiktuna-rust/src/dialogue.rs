mod context {
    use crate::asset::GameAssets;
    use crate::core::behavior::{CrewLossMemory, Reward};
    use crate::core::name::{Name, NameData};
    use std::collections::HashMap;

    #[derive(Default)]
    pub(super) struct TextResolutionContext<'a> {
        resolver_map: HashMap<&'static str, Box<dyn Fn() -> String + 'a>>,
    }

    impl<'a> TextResolutionContext<'a> {
        pub fn add_resolver(&mut self, key: &'static str, resolver: impl Fn() -> String + 'a) {
            self.resolver_map.insert(key, Box::new(resolver));
        }

        pub(super) fn resolve(&self, mut text: &str) -> String {
            let mut result = String::new();
            while !text.is_empty() {
                if let Some(start) = text.find("{")
                    && let Some(length) = text[start..].find("}")
                {
                    result.push_str(&text[..start]);

                    let key = &text[(start + 1)..(start + length)];
                    if let Some(resolver) = self.resolver_map.get(key) {
                        result.push_str(&resolver());
                    } else {
                        eprintln!("Unknown dialogue text key: \"{key}\"");
                        result.push_str("???");
                    }

                    text = text.split_at(start + length + 1).1;
                } else {
                    result.push_str(text);
                    break;
                }
            }
            result
        }
    }

    pub(super) fn setup_context<'a>(
        world: &'a hecs::World,
        speaker: hecs::Entity,
        target: hecs::Entity,
        assets: &'a GameAssets,
    ) -> TextResolutionContext<'a> {
        let mut context = TextResolutionContext::default();

        context.add_resolver("name", move || {
            if let Ok(mut name) = world.get::<&mut Name>(speaker) {
                name.is_known = true;
                name.name.clone()
            } else {
                eprintln!("Missing name for dialogue");
                "???".to_owned()
            }
        });

        context.add_resolver("the_speaker", move || {
            NameData::find(world, speaker, assets).definite()
        });

        context.add_resolver("the_target", move || {
            NameData::find(world, target, assets).definite()
        });

        context.add_resolver("crew_loss_memory_name", move || {
            if let Ok(crew_loss_memory) = world.get::<&CrewLossMemory>(speaker) {
                crew_loss_memory.name.clone()
            } else {
                eprintln!("Missing crew loss memory for dialogue");
                "???".to_owned()
            }
        });

        context.add_resolver("reward", move || {
            if let Ok(reward) = world.get::<&Reward>(speaker) {
                reward.as_text(assets)
            } else {
                eprintln!("Missing reward for dialogue");
                "???".to_owned()
            }
        });

        context
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            asset::GameAssets,
            core::{behavior::CrewLossMemory, name::Name},
        };

        #[test]
        fn resolve() {
            let assets = GameAssets::load().unwrap();
            let mut world = hecs::World::new();
            let speaker = world.spawn((
                Name {
                    name: "foo".to_owned(),
                    is_known: false,
                },
                CrewLossMemory {
                    name: "bar".to_owned(),
                    recent: false,
                },
            ));
            let context = super::setup_context(&world, speaker, speaker, &assets);

            assert_eq!(
                context.resolve("a {name} b {crew_loss_memory_name}"),
                "a foo b bar"
            );
            assert_eq!(context.resolve("{unknown}"), "???");
        }
    }
}

use crate::asset::dialogue::{ConditionedDialogueNode, DialogueData, NextDialogueKind};
use crate::core::area::ShipState;
use crate::core::behavior::{
    self, BackgroundDialogue, Character, CrewLossMemory, Decision, EncounterDialogue,
    GivesHuntRewardData, Passenger, PassengerPhase, Recruitable, Reward, Talk, TalkState,
    TalkedAboutEnoughFuel,
};
use crate::core::name::{Name, NameData};
use crate::core::position::{self, Pos};
use crate::core::status::Health;
use crate::core::store::{self, Shopkeeper};
use crate::core::{self, CrewMember, area, inventory};
use crate::game_loop::GameState;
use crate::{asset, view};
use hecs::{Entity, World};
use rand::seq::{IndexedRandom, IteratorRandom, SliceRandom};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum TalkTopic {
    AskName,
    CompleteHuntQuest,
    CompletePassengerRoute,
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
                trigger_dialogue_by_name("core/ask_name", performer, target, state, view_buffer);
                prompt_npc_dialogue(performer, target, state, view_buffer);
            }
            TalkTopic::CompleteHuntQuest => {
                complete_hunt_quest(performer, target, state, view_buffer)
            }
            TalkTopic::CompletePassengerRoute => {
                let passenger = state
                    .world
                    .get::<&Passenger>(performer)
                    .ok()
                    .as_deref()
                    .cloned();
                if let Some(passenger) = passenger
                    && passenger.phase == PassengerPhase::Leaving
                {
                    trigger_dialogue_by_name(
                        "core/passenger/done",
                        performer,
                        target,
                        state,
                        view_buffer,
                    );
                    if state.world.satisfies::<&Recruitable>(performer)
                        && core::check_crew_size(&state.world).is_ok()
                    {
                        trigger_recruit_request(performer, target, state, view_buffer);
                    } else {
                        trigger_passenger_reward(performer, target, false, state, view_buffer);
                    }
                }
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

            if let Some(dialogue_id) = &gives_hunt_reward.task_dialogue {
                let dialogue_id = dialogue_id.clone();
                drop(gives_hunt_reward);
                trigger_dialogue_by_name(&dialogue_id, npc, crew_member, state, view_buffer);
            } else {
                let message = format!(
                    "{the_npc} asks {the_character} to hunt down {the_target} nearby and offers {a_reward} as reward.",
                    the_npc = NameData::find(&state.world, npc, view_buffer.assets).definite(),
                    the_character =
                        NameData::find(&state.world, crew_member, view_buffer.assets).definite(),
                    the_target = gives_hunt_reward.target_label,
                    a_reward = npc_ref
                        .get::<&Reward>()
                        .map(|reward| reward.as_text(view_buffer.assets))
                        .unwrap_or("???".to_owned()),
                );
                drop(gives_hunt_reward);
                view_buffer.add_change_message(message, state);
            }
        } else {
            let dialogue_id = gives_hunt_reward.already_completed_dialogue.clone();

            if let Some(dialogue_id) = dialogue_id {
                drop(gives_hunt_reward);
                trigger_dialogue_by_name(&dialogue_id, npc, crew_member, state, view_buffer);
            } else {
                let message = format!(
                    "{the_npc} was going to ask for help with {the_target}, but since it has already been taken care of, {the_npc} gives {the_character} {a_reward} as reward.",
                    the_npc = NameData::find(&state.world, npc, view_buffer.assets).definite(),
                    the_character =
                        NameData::find(&state.world, crew_member, view_buffer.assets).definite(),
                    the_target = gives_hunt_reward.target_label,
                    a_reward = npc_ref
                        .get::<&Reward>()
                        .map(|reward| reward.as_text(view_buffer.assets))
                        .unwrap_or("???".to_owned()),
                );
                drop(gives_hunt_reward);
                view_buffer.add_change_message(message, state);
            }

            if let Ok(reward) = state.world.remove_one::<Reward>(npc) {
                reward.give_reward_to(crew_member, &mut state.world);
            }

            let _ = state.world.remove_one::<GivesHuntRewardData>(npc);
        }
    } else {
        drop(gives_hunt_reward);
        if let Some(talk) = npc_ref.get::<&Talk>().map(crate::deref_clone) {
            trigger_dialogue_by_name(&talk.0, npc, crew_member, state, view_buffer);
        } else if npc_ref
            .get::<&Passenger>()
            .is_some_and(|passenger| passenger.phase == PassengerPhase::Requesting)
        {
            trigger_dialogue_by_name(
                "core/passenger/request",
                npc,
                crew_member,
                state,
                view_buffer,
            );
            state
                .world
                .insert_one(crew_member, Decision::Passenger(npc))
                .unwrap();
        } else if let Some(recruitable) = npc_ref.get::<&Recruitable>().map(crate::deref_clone) {
            if recruitable.will_request {
                trigger_recruit_request(npc, crew_member, state, view_buffer);
            } else {
                trigger_dialogue_by_name("core/recruit/hint", npc, crew_member, state, view_buffer);
            }
        } else if npc_ref.has::<Shopkeeper>() {
            store::initiate_trade(crew_member, npc, state, view_buffer);
        }
    }
}

fn trigger_recruit_request(
    npc: Entity,
    crew_member: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer<'_>,
) {
    trigger_dialogue_by_name("core/recruit/request", npc, crew_member, state, view_buffer);
    state
        .world
        .insert_one(crew_member, Decision::Recruit(npc))
        .unwrap();
}

fn trigger_passenger_reward(
    npc: Entity,
    crew_member: Entity,
    is_backup: bool,
    state: &mut GameState,
    view_buffer: &mut view::Buffer<'_>,
) {
    trigger_dialogue_by_name(
        if is_backup {
            "core/passenger/backup_reward"
        } else {
            "core/passenger/reward"
        },
        npc,
        crew_member,
        state,
        view_buffer,
    );
    if let Ok(reward) = state.world.remove_one::<Reward>(npc) {
        reward.give_reward_to(crew_member, &mut state.world);
    }
    view_buffer.messages.add(format!(
        "{} takes their leave.",
        NameData::find(&state.world, npc, view_buffer.assets).definite()
    ));
    state.world.despawn(npc).unwrap();
}

pub(crate) fn trigger_reject_recruitment_request(
    crew_member: Entity,
    npc: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer<'_>,
) {
    trigger_dialogue_by_name(
        "core/recruit/reject_request",
        crew_member,
        npc,
        state,
        view_buffer,
    );

    let passenger = state.world.get::<&Passenger>(npc).ok().as_deref().cloned();
    if let Some(passenger) = passenger
        && passenger.phase == PassengerPhase::Leaving
    {
        trigger_passenger_reward(npc, crew_member, true, state, view_buffer);
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
        target_label,
        reward_dialogue,
        ..
    } = state
        .world
        .get::<&GivesHuntRewardData>(npc)
        .unwrap()
        .deref()
        .clone();

    if let Some(reward_dialogue) = reward_dialogue {
        trigger_dialogue_by_name(&reward_dialogue, crew_member, npc, state, view_buffer);
    } else {
        let message = format!(
            "{the_npc} gives {the_character} {a_reward} as reward for helping out with {the_target}.",
            the_npc = NameData::find(&state.world, npc, view_buffer.assets).definite(),
            the_character =
                NameData::find(&state.world, crew_member, view_buffer.assets).definite(),
            the_target = target_label,
            a_reward = state
                .world
                .get::<&Reward>(npc)
                .map(|reward| reward.as_text(view_buffer.assets))
                .unwrap_or("???".to_owned()),
        );
        view_buffer.add_change_message(message, state);
    }

    if let Ok(reward) = state.world.remove_one::<Reward>(npc) {
        reward.give_reward_to(crew_member, &mut state.world);
    }

    let _ = state.world.remove_one::<GivesHuntRewardData>(npc);
}

#[derive(Debug, Clone, Copy)]
enum ShipDialogue {
    ApproachingFortuna,
    CrewLoss,
    Regular,
}

impl ShipDialogue {
    fn dialogue_id(self) -> &'static str {
        match self {
            ShipDialogue::ApproachingFortuna => "core/on_ship/approaching_fortuna",
            ShipDialogue::CrewLoss => "core/on_ship/crew_loss",
            ShipDialogue::Regular => "core/on_ship/regular",
        }
    }
}

fn pick_ship_dialogue_topic(state: &mut GameState) -> Option<(ShipDialogue, Entity, Entity)> {
    if state.generation_state.locations_before_fortuna() == 0 {
        let mut crew_characters = state
            .world
            .query::<Entity>()
            .with::<(&CrewMember, &Character)>()
            .iter()
            .choose_multiple(&mut state.rng, 2);
        crew_characters.shuffle(&mut state.rng);
        let [character1, character2] = crew_characters[..] else {
            return None;
        };

        return Some((ShipDialogue::ApproachingFortuna, character1, character2));
    }

    if let Some((crew_loss_character, _)) = state
        .world
        .query::<(Entity, &CrewLossMemory)>()
        .with::<(&CrewMember, &Character)>()
        .iter()
        .filter(|(_, crew_loss_memory)| crew_loss_memory.recent)
        .choose(&mut state.rng)
        && let Some(other_character) = state
            .world
            .query::<Entity>()
            .with::<(&CrewMember, &Character)>()
            .iter()
            .filter(|entity| *entity != crew_loss_character)
            .choose(&mut state.rng)
    {
        return Some((ShipDialogue::CrewLoss, crew_loss_character, other_character));
    }

    let mut crew_characters = state
        .world
        .query::<Entity>()
        .with::<(&CrewMember, &Character)>()
        .iter()
        .choose_multiple(&mut state.rng, 2);
    crew_characters.shuffle(&mut state.rng);
    let [character1, character2] = crew_characters[..] else {
        return None;
    };

    Some((ShipDialogue::Regular, character1, character2))
}

pub fn trigger_ship_dialogue(state: &mut GameState, view_buffer: &mut view::Buffer) {
    if let Some((ship_dialogue, character1, character2)) = pick_ship_dialogue_topic(state) {
        let [pos1, pos2] = state
            .world
            .get::<&ShipState>(state.ship_core)
            .unwrap()
            .dialogue_pos;
        state.world.insert_one(character1, pos1).unwrap();
        state.world.insert_one(character2, pos2).unwrap();
        position::turn_towards(&state.world, character1, pos2);
        position::turn_towards(&state.world, character2, pos1);

        trigger_dialogue_by_name(
            ship_dialogue.dialogue_id(),
            character1,
            character2,
            state,
            view_buffer,
        );

        view_buffer.capture_view(state, false);
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
        .query::<(Entity, &Pos)>()
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
        .query::<(Entity, &Pos)>()
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
            .query_mut::<(Entity, &Pos)>()
            .with::<(&CrewMember, &Character)>()
            .into_iter()
            .filter(|&(entity, pos)| entity != state.controlled && pos.is_in(player_pos.get_area()))
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        let crew = state.world.get::<&CrewMember>(state.controlled).unwrap().0;
        if !state.world.satisfies::<&TalkedAboutEnoughFuel>(crew)
            && area::fuel_needed_to_launch(&state.world).is_some_and(|fuel_amount| {
                fuel_amount <= inventory::fuel_cans_held_by_crew(&state.world, &[])
            })
        {
            state.world.insert_one(crew, TalkedAboutEnoughFuel).unwrap();
            if let Some(&speaker) = possible_speakers.choose(&mut state.rng) {
                trigger_dialogue_by_name(
                    "core/obtained_enough_fuel",
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
                    "core/badly_hurt_after_battle",
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

    if target_pos.is_in(speaker_pos.get_area()) && state.world.satisfies::<&Character>(target) {
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
        .query_mut::<(Entity, &Pos)>()
        .with::<(&CrewMember, &Character)>()
        .into_iter()
        .find(|&(entity, pos)| entity != state.controlled && pos.is_in(player_pos.get_area()))
        .map(|(entity, _)| entity)
    else {
        return;
    };

    trigger_dialogue_by_name(
        "core/landing/start",
        speaker,
        state.controlled,
        state,
        view_buffer,
    );
}

pub fn trigger_dialogue_by_name(
    name: &str,
    speaker: Entity,
    target: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    match asset::dialogue::load_dialogue_data(name) {
        Ok(dialogue_data) => run_dialogue(&dialogue_data, speaker, target, state, view_buffer),
        Err(error) => {
            println!("Failed to load dialogue {name}: {error}");
            view_buffer
                .messages
                .add(format!("System: Unable to load dialogue \"{name}\"."));
        }
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
    let context = context::setup_context(world, speaker, target, view_buffer.assets);
    let target_pos = *world.get::<&Pos>(target).unwrap();
    position::turn_towards(world, speaker, target_pos);

    let message = context.resolve(&dialogue_node.message);

    view_buffer.push_dialogue(world, speaker, dialogue_node.expression, message);

    if let Some(reply) = &dialogue_node.reply
        && let Some(reply_node) = reply.select_node(target, speaker, state)
    {
        run_dialogue_node(reply_node, target, speaker, state, view_buffer);
    }
}

fn run_dialogue(
    dialogue_data: &DialogueData,
    speaker: Entity,
    target: Entity,
    state: &mut GameState,
    view_buffer: &mut view::Buffer,
) {
    if let Some(dialogue) = dialogue_data.dialogue.select_node(speaker, target, state) {
        view_buffer.capture_view_before_dialogue(state);
        run_dialogue_node(dialogue, speaker, target, state, view_buffer);
    } else if !dialogue_data.description.is_empty() {
        let context = context::setup_context(&state.world, speaker, target, view_buffer.assets);

        view_buffer
            .messages
            .add(context.resolve(&dialogue_data.description));
    }

    if let Some(effect) = &dialogue_data.effect {
        effect.apply(speaker, target, &mut state.world);
    }

    if let Some(next) = &dialogue_data.next.iter().find(|next| match next.kind {
        NextDialogueKind::Response => next.condition.test(target, speaker, state),
        NextDialogueKind::Continuation => next.condition.test(speaker, target, state),
    }) {
        let (next_speaker, next_target) = match next.kind {
            NextDialogueKind::Response => (target, speaker),
            NextDialogueKind::Continuation => (speaker, target),
        };
        match &next.node {
            asset::dialogue::RefOrData::Ref(dialogue_id) => {
                trigger_dialogue_by_name(dialogue_id, next_speaker, next_target, state, view_buffer)
            }
            asset::dialogue::RefOrData::Data(dialogue_data) => {
                run_dialogue(dialogue_data, next_speaker, next_target, state, view_buffer)
            }
        }
    }
}
