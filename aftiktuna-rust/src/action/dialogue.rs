use crate::action::{self, Context, Error};
use crate::core::display::DialogueExpression;
use crate::core::name::{Name, NameData};
use crate::core::position::{Direction, Pos};
use crate::core::{self, CrewMember, Recruitable, Waiting, area, position, status};
use hecs::Entity;

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
            crate::dialogue::talk_dialogue(
                performer,
                target,
                &mut state.world,
                view_context.view_buffer,
            );
            None
        },
    )
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
            view_context.view_buffer.push_dialogue(
                &state.world,
                performer,
                DialogueExpression::Neutral,
                "Hi! Do you want to join me in the search for Fortuna?",
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
                view_context.view_buffer.push_dialogue(
                    &state.world,
                    target,
                    DialogueExpression::Excited,
                    format!("Sure, I'll join you! My name is {name_string}."),
                );
            } else {
                view_context.view_buffer.push_dialogue(
                    &state.world,
                    target,
                    DialogueExpression::Excited,
                    "Sure, I'll join you!",
                );
            }

            state.world.remove_one::<Recruitable>(target).unwrap();
            let name = NameData::find(&state.world, target).definite();
            state.world.insert_one(target, CrewMember(crew)).unwrap();

            view_context
                .view_buffer
                .add_change_message(format!("{name} joined the crew!"), state);
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
            view_context.view_buffer.push_dialogue(
                &state.world,
                performer,
                DialogueExpression::Neutral,
                "Please wait here for now.",
            );
            view_context.view_buffer.push_dialogue(
                &state.world,
                target,
                DialogueExpression::Neutral,
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
                view_context.view_buffer.push_dialogue(
                    &state.world,
                    performer,
                    DialogueExpression::Neutral,
                    "Please wait at the ship.",
                );

                view_context.view_buffer.push_dialogue(
                    &state.world,
                    target,
                    DialogueExpression::Neutral,
                    "Sure thing. I will stay here for now.",
                );
            } else {
                view_context.view_buffer.push_dialogue(
                    &state.world,
                    performer,
                    DialogueExpression::Neutral,
                    "Please go back and wait at the ship.",
                );

                view_context.view_buffer.push_dialogue(
                    &state.world,
                    target,
                    DialogueExpression::Neutral,
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
            view_context.view_buffer.push_dialogue(
                &state.world,
                performer,
                DialogueExpression::Neutral,
                "Time to go, please follow me.",
            );
            view_context.view_buffer.push_dialogue(
                &state.world,
                target,
                DialogueExpression::Neutral,
                "Alright, let's go!",
            );

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
            context.state,
        );
        Ok(action::Success)
    })
}
