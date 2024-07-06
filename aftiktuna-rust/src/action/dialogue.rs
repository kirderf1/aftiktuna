use crate::action::{self, Context, Error};
use crate::core::inventory::Held;
use crate::core::name::{Name, NameData};
use crate::core::position::{Direction, Pos};
use crate::core::status::Health;
use crate::core::{
    self, area, position, status, CrewMember, GivesHuntReward, Recruitable, RepeatingAction, Tag,
    Waiting,
};
use hecs::{Entity, World};

use super::DialogueContext;

pub(super) fn talk_to(context: Context, performer: Entity, target: Entity) -> action::Result {
    let Context {
        state,
        mut dialogue_context,
    } = context;
    let world = &state.world;
    if !status::is_alive(target, world) {
        return action::silent_ok();
    }

    let performer_pos = *world.get::<&Pos>(performer).unwrap();
    let target_pos = *world.get::<&Pos>(target).unwrap();

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(|blockage| blockage.into_message(world))?;

    dialogue_context.capture_frame_for_dialogue(state);
    let world = &mut state.world;

    if performer_pos != target_pos {
        movement.perform(world).unwrap();
        world
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    talk_dialogue(performer, target, world, dialogue_context);

    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

fn talk_dialogue(
    performer: Entity,
    target: Entity,
    world: &mut World,
    mut context: DialogueContext,
) {
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
        regular_greeting(performer, target, world, &mut context);
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
                item_reward,
                ..
            } = world.remove_one::<GivesHuntReward>(target).unwrap();

            context.add_dialogue(world, target, format!("\"{reward_message}\""));

            for item_type in item_reward {
                item_type.spawn(world, Held::in_inventory(performer));
            }
        }
    } else if target_ref.has::<Recruitable>() {
        context.add_dialogue(
            world,
            target,
            "\"I wish I could leave this place and go on an adventure.\"",
        );
    }
}

fn regular_greeting(
    performer: Entity,
    target: Entity,
    world: &World,
    context: &mut DialogueContext,
) {
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
    let Context {
        state,
        mut dialogue_context,
    } = context;
    let world = &state.world;
    let target_pos = *world.get::<&Pos>(target).unwrap();
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let crew_size = world.query::<&CrewMember>().iter().count();
    if crew_size >= core::CREW_SIZE_LIMIT {
        return Err(Error::private(
            "There is not enough room for another crew member.",
        ));
    }

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(|blockage| blockage.into_message(world))?;

    dialogue_context.capture_frame_for_dialogue(state);
    let world = &mut state.world;

    movement.perform(world).unwrap();

    dialogue_context.add_dialogue(
        world,
        performer,
        "\"Hi! Do you want to join me in the search for Fortuna?\"",
    );
    if world
        .get::<&Name>(target)
        .ok()
        .map_or(false, |name| !name.is_known)
    {
        let name_string = {
            let mut name_ref = world.get::<&mut Name>(target).unwrap();
            name_ref.is_known = true;
            name_ref.name.clone()
        };
        dialogue_context.add_dialogue(
            world,
            target,
            format!("\"Sure, I'll join you! My name is {name_string}.\""),
        );
    } else {
        dialogue_context.add_dialogue(world, target, "\"Sure, I'll join you!\"");
    }

    world.remove_one::<Recruitable>(target).unwrap();
    let name = NameData::find(world, target).definite();
    world.insert_one(target, CrewMember(crew)).unwrap();

    action::ok(format!("{name} joined the crew!"))
}

pub(super) fn tell_to_wait(context: Context, performer: Entity, target: Entity) -> action::Result {
    let Context {
        state,
        mut dialogue_context,
    } = context;
    if !status::is_alive(target, &state.world) || state.world.satisfies::<&Waiting>(target).unwrap()
    {
        return action::silent_ok();
    }

    let performer_pos = *state.world.get::<&Pos>(performer).unwrap();
    let target_pos = *state.world.get::<&Pos>(target).unwrap();

    dialogue_context.capture_frame_for_dialogue(state);
    let world = &mut state.world;

    if performer_pos != target_pos {
        world
            .insert_one(performer, Direction::between(performer_pos, target_pos))
            .unwrap();
        world
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    dialogue_context.add_dialogue(world, performer, "Please wait here for now.");
    dialogue_context.add_dialogue(
        world,
        target,
        "Sure thing. Just tell me when I should follow along again.",
    );

    world.insert_one(target, Waiting).unwrap();

    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

pub(super) fn tell_to_wait_at_ship(
    context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    let Context {
        state,
        mut dialogue_context,
    } = context;
    if !status::is_alive(target, &state.world) {
        return action::silent_ok();
    }

    let performer_pos = *state.world.get::<&Pos>(performer).unwrap();
    let target_pos = *state.world.get::<&Pos>(target).unwrap();

    if area::is_ship(target_pos.get_area(), &state.world) {
        return Err(Error::private("They are already at the ship."));
    }

    dialogue_context.capture_frame_for_dialogue(state);
    let world = &mut state.world;

    if performer_pos != target_pos {
        world
            .insert_one(performer, Direction::between(performer_pos, target_pos))
            .unwrap();
        world
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    dialogue_context.add_dialogue(world, performer, "Please go back and wait at the ship.");
    dialogue_context.add_dialogue(
        world,
        target,
        "Sure thing. I will go and wait at the ship for now.",
    );

    world
        .insert(target, (Waiting, RepeatingAction::GoToShip))
        .unwrap();

    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

pub(super) fn tell_to_follow(
    context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    let Context {
        state,
        mut dialogue_context,
    } = context;
    if !status::is_alive(target, &state.world)
        || !state.world.satisfies::<&Waiting>(target).unwrap()
    {
        return action::silent_ok();
    }

    let performer_pos = *state.world.get::<&Pos>(performer).unwrap();
    let target_pos = *state.world.get::<&Pos>(target).unwrap();

    dialogue_context.capture_frame_for_dialogue(state);
    let world = &mut state.world;

    if performer_pos != target_pos {
        world
            .insert_one(performer, Direction::between(performer_pos, target_pos))
            .unwrap();
        world
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    dialogue_context.add_dialogue(world, performer, "Time to go, please follow me.");
    dialogue_context.add_dialogue(world, target, "Alright, let's go!");

    world.remove_one::<Waiting>(target).unwrap();

    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}
