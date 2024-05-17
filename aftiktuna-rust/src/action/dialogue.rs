use crate::action;
use crate::action::Context;
use crate::core::position::{Blockage, Direction, Pos};
use crate::core::status::Health;
use crate::core::{area, position, status, CrewMember, GoingToShip, Recruitable, Waiting};
use crate::view::name::{Name, NameData};
use hecs::Entity;

pub(super) fn talk_to(mut context: Context, performer: Entity, target: Entity) -> action::Result {
    let world = context.mut_world();
    if !status::is_alive(target, world) {
        return action::silent_ok();
    }

    let performer_pos = *world.get::<&Pos>(performer).unwrap();
    let target_pos = *world.get::<&Pos>(target).unwrap();

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(Blockage::into_message)?;

    context.capture_frame_for_dialogue();

    if performer_pos != target_pos {
        movement.perform(context.mut_world()).unwrap();
        context
            .mut_world()
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    talk_dialogue(&mut context, performer, target);

    let world = context.mut_world();
    let performer_name = NameData::find(world, performer).definite();
    let target_name = NameData::find(world, target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

fn talk_dialogue(context: &mut Context, performer: Entity, target: Entity) {
    if context
        .mut_world()
        .get::<&Name>(target)
        .ok()
        .map_or(false, |name| !name.is_known)
    {
        let mut name_ref = context.mut_world().get::<&mut Name>(target).unwrap();
        name_ref.is_known = true;
        let name_string = name_ref.name.clone();
        drop(name_ref);
        context.add_dialogue(performer, "\"Hi! What is your name?\"");
        context.add_dialogue(target, format!("\"My name is {name_string}.\""));
    } else if context.mut_world().get::<&Recruitable>(target).is_ok() {
        context.add_dialogue(performer, "\"Hi!\"");
        context.add_dialogue(
            target,
            "\"Hello! I wish I could leave this place and go on an adventure.\"",
        );
    } else if context.mut_world().get::<&Waiting>(target).is_ok() {
        context.add_dialogue(performer, "\"Hi!\"");
        context.add_dialogue(
            target,
            "\"Hello! I'll continue to wait here until you tell me otherwise.\"",
        );
    } else if context
        .mut_world()
        .get::<&Health>(target)
        .map_or(false, |health| health.as_fraction() < 0.5)
    {
        context.add_dialogue(performer, "\"Hi!\"");
        context.add_dialogue(target, "\"Hello! I'm not doing too well right now. Perhaps I should stay behind if we will be exploring anything more.\"");
    } else {
        context.add_dialogue(performer, "\"Hi!\"");
        context.add_dialogue(target, "\"Hello!\"");
    }
}

pub(super) fn recruit(mut context: Context, performer: Entity, target: Entity) -> action::Result {
    let world = context.mut_world();
    let target_pos = *world.get::<&Pos>(target).unwrap();
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let crew_size = world.query::<&CrewMember>().iter().count();
    if crew_size >= 2 {
        return Err("There is not enough room for another crew member.".to_string());
    }

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(Blockage::into_message)?;

    context.capture_frame_for_dialogue();

    movement.perform(context.mut_world()).unwrap();

    context.add_dialogue(
        performer,
        "\"Hi! Do you want to join me in the search for Fortuna?\"",
    );
    if context
        .mut_world()
        .get::<&Name>(target)
        .ok()
        .map_or(false, |name| !name.is_known)
    {
        let mut name_ref = context.mut_world().get::<&mut Name>(target).unwrap();
        name_ref.is_known = true;
        let name_string = name_ref.name.clone();
        drop(name_ref);
        context.add_dialogue(
            target,
            format!("\"Sure, I'll join you! My name is {name_string}.\""),
        );
    } else {
        context.add_dialogue(target, "\"Sure, I'll join you!\"");
    }

    let world = context.mut_world();
    world.remove_one::<Recruitable>(target).unwrap();
    let name = NameData::find(world, target).definite();
    world.insert_one(target, CrewMember(crew)).unwrap();

    action::ok(format!("{name} joined the crew!"))
}

pub(super) fn tell_to_wait(
    mut context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    if !status::is_alive(target, context.mut_world())
        || context.mut_world().satisfies::<&Waiting>(target).unwrap()
    {
        return action::silent_ok();
    }

    let performer_pos = *context.mut_world().get::<&Pos>(performer).unwrap();
    let target_pos = *context.mut_world().get::<&Pos>(target).unwrap();

    context.capture_frame_for_dialogue();

    if performer_pos != target_pos {
        context
            .mut_world()
            .insert_one(performer, Direction::between(performer_pos, target_pos))
            .unwrap();
        context
            .mut_world()
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    context.add_dialogue(performer, "Please wait here for now.");
    context.add_dialogue(
        target,
        "Sure thing. Just tell me when I should follow along again.",
    );

    context.mut_world().insert_one(target, Waiting).unwrap();

    let performer_name = NameData::find(context.mut_world(), performer).definite();
    let target_name = NameData::find(context.mut_world(), target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

pub(super) fn tell_to_wait_at_ship(
    mut context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    if !status::is_alive(target, context.mut_world())
        || context
            .mut_world()
            .satisfies::<&GoingToShip>(target)
            .unwrap()
    {
        return action::silent_ok();
    }

    let performer_pos = *context.mut_world().get::<&Pos>(performer).unwrap();
    let target_pos = *context.mut_world().get::<&Pos>(target).unwrap();

    if area::is_ship(target_pos.get_area(), context.mut_world()) {
        return Err("They are already at the ship.".to_string());
    }

    context.capture_frame_for_dialogue();

    if performer_pos != target_pos {
        context
            .mut_world()
            .insert_one(performer, Direction::between(performer_pos, target_pos))
            .unwrap();
        context
            .mut_world()
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    context.add_dialogue(performer, "Please go back and wait at the ship.");
    context.add_dialogue(
        target,
        "Sure thing. I will go and wait at the ship for now.",
    );

    context
        .mut_world()
        .insert(target, (Waiting, GoingToShip))
        .unwrap();

    let performer_name = NameData::find(context.mut_world(), performer).definite();
    let target_name = NameData::find(context.mut_world(), target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}

pub(super) fn tell_to_follow(
    mut context: Context,
    performer: Entity,
    target: Entity,
) -> action::Result {
    if !status::is_alive(target, context.mut_world())
        || !context.mut_world().satisfies::<&Waiting>(target).unwrap()
    {
        return action::silent_ok();
    }

    let performer_pos = *context.mut_world().get::<&Pos>(performer).unwrap();
    let target_pos = *context.mut_world().get::<&Pos>(target).unwrap();

    context.capture_frame_for_dialogue();

    if performer_pos != target_pos {
        context
            .mut_world()
            .insert_one(performer, Direction::between(performer_pos, target_pos))
            .unwrap();
        context
            .mut_world()
            .insert_one(target, Direction::between(target_pos, performer_pos))
            .unwrap();
    }

    context.add_dialogue(performer, "Time to go, please follow me.");
    context.add_dialogue(target, "Alright, let's go!");

    context.mut_world().remove_one::<Waiting>(target).unwrap();

    let performer_name = NameData::find(context.mut_world(), performer).definite();
    let target_name = NameData::find(context.mut_world(), target).definite();
    action::ok(format!(
        "{performer_name} finishes talking with {target_name}."
    ))
}
