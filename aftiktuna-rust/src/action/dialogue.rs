use crate::action::{self, Context, Error};
use crate::core::behavior::{Hostile, Recruitable, Waiting};
use crate::core::name::NameData;
use crate::core::position::{Placement, PlacementQuery, Pos};
use crate::core::status::Morale;
use crate::core::{self, CrewMember, position, status};
use crate::dialogue::{self, TalkTopic};
use hecs::Entity;

#[derive(Clone, Debug)]
pub struct TalkAction {
    pub target: Entity,
    pub topic: TalkTopic,
}

impl From<TalkAction> for super::Action {
    fn from(value: TalkAction) -> Self {
        Self::TalkTo(value)
    }
}

impl TalkAction {
    pub(super) fn run(self, context: Context, performer: Entity) -> action::Result {
        let Self { target, topic } = self;
        if !status::is_alive(target, &context.state.world) {
            return Ok(action::Success);
        }
        if context.state.world.satisfies::<&Hostile>(target).unwrap() {
            return Err(Error::private(format!(
                "{the_target} is not interested in talking.",
                the_target = NameData::find(
                    &context.state.world,
                    target,
                    context.view_context.view_buffer.assets,
                )
                .definite(),
            )));
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
                topic.perform(performer, target, state, view_context.view_buffer);
                None
            },
        )
    }
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
            if state.world.satisfies::<&Recruitable>(target).unwrap() {
                dialogue::trigger_dialogue_by_name(
                    "recruit",
                    performer,
                    target,
                    state,
                    view_context.view_buffer,
                );

                if let Ok(mut morale) = state.world.get::<&mut Morale>(target) {
                    morale.journey_start_effect();
                }
                for (_, morale) in state.world.query_mut::<&mut Morale>().with::<&CrewMember>() {
                    morale.new_crew_member_effect();
                }
                state.world.remove_one::<Recruitable>(target).unwrap();
                let name = NameData::find(&state.world, target, view_context.view_buffer.assets)
                    .definite();
                state.world.insert_one(target, CrewMember(crew)).unwrap();

                view_context
                    .view_buffer
                    .add_change_message(format!("{name} joined the crew!"), state);
                Some(Ok(action::Success))
            } else {
                dialogue::trigger_dialogue_by_name(
                    "recruit_fail",
                    performer,
                    target,
                    state,
                    view_context.view_buffer,
                );
                None
            }
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
            dialogue::trigger_dialogue_by_name(
                "tell_to/wait",
                performer,
                target,
                state,
                view_context.view_buffer,
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
            dialogue::trigger_dialogue_by_name(
                "tell_to/wait_at_ship",
                performer,
                target,
                state,
                view_context.view_buffer,
            );

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
            dialogue::trigger_dialogue_by_name(
                "tell_to/follow",
                performer,
                target,
                state,
                view_context.view_buffer,
            );

            state.world.remove_one::<Waiting>(target).unwrap();

            None
        },
    )
}

pub(super) fn full_dialogue_action(
    mut context: Context,
    performer: Entity,
    target: Entity,
    move_adjacent: bool,
    dialogue: impl FnOnce(&mut Context) -> Option<action::Result>,
) -> action::Result {
    let assets = context.view_context.view_buffer.assets;
    let performer_pos = *context.state.world.get::<&Pos>(performer).unwrap();
    let target_placement = Placement::from(
        context
            .state
            .world
            .query_one_mut::<PlacementQuery>(target)
            .unwrap(),
    );

    if !performer_pos.is_in(target_placement.area()) {
        let performer_name = NameData::find(&context.state.world, performer, assets).definite();
        let target_name = NameData::find(&context.state.world, target, assets).definite();
        return Err(Error::private(format!(
            "{target_name} left before {performer_name} could talk to them.",
        )));
    }

    let movement = if move_adjacent {
        let movement = position::prepare_move_adjacent_placement(
            &context.state.world,
            performer,
            target_placement,
        )
        .map_err(|blockage| blockage.into_message(&context.state.world, assets))?;
        Some(movement)
    } else {
        None
    };

    context
        .view_context
        .capture_frame_for_dialogue(context.state);

    if performer_pos != target_placement.pos {
        if let Some(movement) = movement {
            movement.perform(&mut context.state.world).unwrap();
        } else {
            position::turn_towards(&context.state.world, performer, target_placement.pos);
        }
        position::turn_towards(&context.state.world, target, performer_pos);
    }

    let result = dialogue(&mut context);

    result.unwrap_or_else(|| {
        let performer_name = NameData::find(&context.state.world, performer, assets).definite();
        let target_name = NameData::find(&context.state.world, target, assets).definite();
        context.view_context.add_message_at(
            performer_pos.get_area(),
            format!("{performer_name} finishes talking with {target_name}."),
            context.state,
        );
        Ok(action::Success)
    })
}
