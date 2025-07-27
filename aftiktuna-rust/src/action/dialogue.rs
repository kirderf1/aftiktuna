use crate::action::{self, Context, Error, ViewContext};
use crate::core::name::{Name, NameData};
use crate::core::position::{Direction, Pos};
use crate::core::status::Health;
use crate::core::{
    self, CrewMember, GivesHuntReward, Recruitable, Tag, Waiting, area, position, status,
};
use hecs::{Entity, World};

pub(super) fn talk_to(context: Context, performer: Entity, target: Entity) -> action::Result {
    if !status::is_alive(target, &context.state.world) {
        return Ok(action::Success);
    }

    full_dialogue_action(
        context,
        performer,
        target,
        true,
        |Context {
             state,
             view_context,
         }| {
            talk_dialogue(performer, target, &mut state.world, view_context);
            None
        },
    )
}

fn talk_dialogue(performer: Entity, target: Entity, world: &mut World, context: &mut ViewContext) {
    let target_ref = world.entity(target).unwrap();
    if target_ref
        .get::<&Name>()
        .map_or(false, |name| !name.is_known)
    {
        let name_string = {
            let mut name_ref = target_ref.get::<&mut Name>().unwrap();
            name_ref.is_known = true;
            name_ref.name.clone()
        };
        context.add_dialogue(world, performer, "\"Hi! What is your name?\"");
        context.add_dialogue(world, target, format!("\"My name is {name_string}.\""));
    } else {
        regular_greeting(performer, target, world, context);
    }

    let gives_hunt_reward = target_ref.get::<&GivesHuntReward>();
    if gives_hunt_reward.is_some() {
        let gives_hunt_reward = gives_hunt_reward.unwrap();
        if any_alive_with_tag(&gives_hunt_reward.target_tag, world) {
            let message = format!("\"{}\"", &gives_hunt_reward.task_message);
            context.add_dialogue(world, target, message);
        } else {
            drop(gives_hunt_reward);
            let GivesHuntReward {
                reward_message,
                reward,
                ..
            } = world.remove_one::<GivesHuntReward>(target).unwrap();

            context.add_dialogue(world, target, format!("\"{reward_message}\""));

            reward.give_reward_to(performer, world);
        }
    } else if target_ref.has::<Recruitable>() {
        context.add_dialogue(
            world,
            target,
            "\"I wish I could leave this place and go on an adventure.\"",
        );
    }
}

fn regular_greeting(performer: Entity, target: Entity, world: &World, context: &mut ViewContext) {
    let target_ref = world.entity(target).unwrap();

    context.add_dialogue(world, performer, "\"Hi!\"");

    if target_ref
        .get::<&GivesHuntReward>()
        .map_or(false, |gives_hunt_reward| {
            any_alive_with_tag(&gives_hunt_reward.target_tag, world)
        })
    {
        context.add_dialogue(
            world,
            target,
            "\"Hello! I have a bit of a problem at the moment.\"",
        );
    } else if target_ref.has::<Waiting>() {
        context.add_dialogue(
            world,
            target,
            "\"Hello! I'll continue to wait here until you tell me otherwise.\"",
        );
    } else if target_ref
        .get::<&Health>()
        .map_or(false, |health| health.is_badly_hurt())
    {
        context.add_dialogue(world, target, "\"Hello! I'm not doing too well right now. Perhaps I should stay behind if we will be exploring anything more.\"");
    } else {
        context.add_dialogue(world, target, "\"Hello!\"");
    }
}

fn any_alive_with_tag(target_tag: &Tag, world: &World) -> bool {
    world
        .query::<(&Health, &Tag)>()
        .iter()
        .any(|(_, (health, tag))| health.is_alive() && target_tag == tag)
}

pub(super) fn recruit(context: Context, performer: Entity, target: Entity) -> action::Result {
    let crew = context.state.world.get::<&CrewMember>(performer).unwrap().0;
    let crew_size = context.state.world.query::<&CrewMember>().iter().count();
    if crew_size >= core::CREW_SIZE_LIMIT {
        return Err(Error::private(
            "There is not enough room for another crew member.",
        ));
    }

    full_dialogue_action(
        context,
        performer,
        target,
        true,
        |Context {
             state,
             view_context,
         }| {
            view_context.add_dialogue(
                &state.world,
                performer,
                "\"Hi! Do you want to join me in the search for Fortuna?\"",
            );
            if state
                .world
                .get::<&Name>(target)
                .ok()
                .is_some_and(|name| !name.is_known)
            {
                let name_string = {
                    let mut name_ref = state.world.get::<&mut Name>(target).unwrap();
                    name_ref.is_known = true;
                    name_ref.name.clone()
                };
                view_context.add_dialogue(
                    &state.world,
                    target,
                    format!("\"Sure, I'll join you! My name is {name_string}.\""),
                );
            } else {
                view_context.add_dialogue(&state.world, target, "\"Sure, I'll join you!\"");
            }

            state.world.remove_one::<Recruitable>(target).unwrap();
            let name = NameData::find(&state.world, target).definite();
            state.world.insert_one(target, CrewMember(crew)).unwrap();

            view_context.add_message_at(
                state.world.get::<&Pos>(performer).unwrap().get_area(),
                format!("{name} joined the crew!"),
            );
            Some(Ok(action::Success))
        },
    )
}

pub(super) fn tell_to_wait(context: Context, performer: Entity, target: Entity) -> action::Result {
    if !status::is_alive(target, &context.state.world)
        || context.state.world.satisfies::<&Waiting>(target).unwrap()
    {
        return Ok(action::Success);
    }

    full_dialogue_action(
        context,
        performer,
        target,
        false,
        |Context {
             state,
             view_context,
         }| {
            view_context.add_dialogue(&state.world, performer, "Please wait here for now.");
            view_context.add_dialogue(
                &state.world,
                target,
                "Sure thing. Just tell me when I should follow along again.",
            );

            state
                .world
                .insert_one(target, Waiting { at_ship: false })
                .unwrap();

            None
        },
    )
}

pub(super) fn tell_to_wait_at_ship(
    context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    if !status::is_alive(target, &context.state.world) {
        return Ok(action::Success);
    }

    full_dialogue_action(
        context,
        performer,
        target,
        false,
        |Context {
             state,
             view_context,
         }| {
            let target_pos = *state.world.get::<&Pos>(target).unwrap();
            if area::is_in_ship(target_pos, &state.world) {
                view_context.add_dialogue(&state.world, performer, "Please wait at the ship.");

                view_context.add_dialogue(
                    &state.world,
                    target,
                    "Sure thing. I will stay here for now.",
                );
            } else {
                view_context.add_dialogue(
                    &state.world,
                    performer,
                    "Please go back and wait at the ship.",
                );

                view_context.add_dialogue(
                    &state.world,
                    target,
                    "Sure thing. I will go and wait at the ship for now.",
                );
            }

            state
                .world
                .insert_one(target, Waiting { at_ship: true })
                .unwrap();

            None
        },
    )
}

pub(super) fn tell_to_follow(
    context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    if !status::is_alive(target, &context.state.world)
        || !context.state.world.satisfies::<&Waiting>(target).unwrap()
    {
        return Ok(action::Success);
    }

    full_dialogue_action(
        context,
        performer,
        target,
        false,
        |Context {
             state,
             view_context,
         }| {
            view_context.add_dialogue(&state.world, performer, "Time to go, please follow me.");
            view_context.add_dialogue(&state.world, target, "Alright, let's go!");

            state.world.remove_one::<Waiting>(target).unwrap();

            None
        },
    )
}

fn full_dialogue_action(
    mut context: Context,
    performer: Entity,
    target: Entity,
    move_adjacent: bool,
    dialogue: impl FnOnce(&mut Context) -> Option<action::Result>,
) -> action::Result {
    let performer_pos = *context.state.world.get::<&Pos>(performer).unwrap();
    let target_pos = *context.state.world.get::<&Pos>(target).unwrap();

    let movement = if move_adjacent {
        let movement = position::prepare_move_adjacent(&context.state.world, performer, target_pos)
            .map_err(|blockage| blockage.into_message(&context.state.world))?;
        Some(movement)
    } else {
        None
    };

    context
        .view_context
        .capture_frame_for_dialogue(context.state);

    if performer_pos != target_pos {
        if let Some(movement) = movement {
            movement.perform(&mut context.state.world).unwrap();
        } else {
            context
                .state
                .world
                .insert_one(performer, Direction::between(performer_pos, target_pos))
                .unwrap();
        }
        context
            .state
            .world
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    let result = dialogue(&mut context);

    result.unwrap_or_else(|| {
        let performer_name = NameData::find(&context.state.world, performer).definite();
        let target_name = NameData::find(&context.state.world, target).definite();
        context.view_context.add_message_at(
            performer_pos.get_area(),
            format!("{performer_name} finishes talking with {target_name}."),
        );
        Ok(action::Success)
    })
}
