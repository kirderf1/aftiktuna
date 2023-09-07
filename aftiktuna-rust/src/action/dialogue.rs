use crate::action;
use crate::action::{Context, CrewMember, Recruitable};
use crate::core::position::{Blockage, Pos};
use crate::core::{position, status};
use crate::view::name::{Name, NameData};
use crate::view::Symbol;
use hecs::Entity;

pub(super) fn talk_to(mut context: Context, performer: Entity, target: Entity) -> action::Result {
    let world = context.mut_world();
    if !status::is_alive(target, world) {
        return action::silent_ok();
    }
    let target_pos = *world.get::<&Pos>(target).unwrap();

    let movement = position::prepare_move_adjacent(world, performer, target_pos)
        .map_err(Blockage::into_message)?;

    context.capture_frame_for_dialogue();

    movement.perform(context.mut_world()).unwrap();

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
    context.add_dialogue(target, "\"Sure, I'll join you!\"");

    let world = context.mut_world();
    world.remove_one::<Recruitable>(target).unwrap();
    if let Ok(mut name) = world.get::<&mut Name>(target) {
        name.is_known = true;
    }
    let name = NameData::find(world, target).definite();
    world
        .insert(target, (Symbol::from_name(&name), CrewMember(crew)))
        .unwrap();

    action::ok(format!("{name} joined the crew!"))
}
