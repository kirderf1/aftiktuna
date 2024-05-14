use crate::action::{Context, CrewMember};
use crate::ai::Intention;
use crate::core::item::{Keycard, Tool};
use crate::core::position::{Blockage, Pos};
use crate::core::{area, inventory, position, BlockType, Door, DoorKind, GoingToShip, IsCut};
use crate::game_loop::GameState;
use crate::view::name::NameData;
use crate::{action, core};
use hecs::{Entity, World};
use std::collections::HashSet;
use std::ops::Deref;

fn move_message(door_kind: DoorKind, performer: &str) -> String {
    match door_kind {
        DoorKind::Door => format!("{performer} entered the door into a new area."),
        DoorKind::Path => format!("{performer} followed the path to a new area."),
    }
}

fn check_tool_for_forcing(
    block_type: BlockType,
    world: &World,
    performer: Entity,
    performer_name: &str,
) -> Result<Tool, String> {
    for tool in block_type.usable_tools() {
        if inventory::is_holding_tool(world, performer, tool) {
            return Ok(tool);
        }
    }
    match block_type {
        BlockType::Stuck => Err(format!(
            "{performer_name} needs some sort of tool to force the door open.",
        )),
        BlockType::Sealed | BlockType::Locked => Err(format!(
            "{performer_name} needs some sort of tool to break the door open.",
        )),
    }
}

pub(super) fn enter_door(state: &mut GameState, aftik: Entity, door: Entity) -> action::Result {
    let world = &mut state.world;
    let aftik_name = NameData::find(world, aftik).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{aftik_name} lost track of the door."))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(aftik).map(|pos| pos.get_area()) {
        return Err(format!("{aftik_name} cannot reach the door from here."));
    }

    position::move_to(world, aftik, door_pos)?;

    let door_data = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())
        .map(|door| door.deref().clone())?;

    let used_keycard = if let Ok(block_type) = world
        .get::<&BlockType>(door_data.door_pair)
        .map(|block_type| *block_type)
    {
        if block_type == BlockType::Locked && inventory::is_holding::<&Keycard>(world, aftik) {
            true
        } else {
            on_door_failure(state, aftik, door, block_type);
            return Err(format!("The door is {}.", block_type.description()));
        }
    } else {
        false
    };

    world.insert_one(aftik, door_data.destination).unwrap();
    let areas = vec![door_pos.get_area(), door_data.destination.get_area()];
    if used_keycard {
        action::ok_at(
            format!(
                "Using their keycard, {}",
                move_message(door_data.kind, &aftik_name),
            ),
            areas,
        )
    } else {
        action::ok_at(move_message(door_data.kind, &aftik_name), areas)
    }
}

pub(super) fn force_door(
    mut context: Context,
    performer: Entity,
    door: Entity,
    assisting: bool,
) -> action::Result {
    let world = context.mut_world();
    let performer_name = NameData::find(world, performer).definite();
    let door_pos = *world
        .get::<&Pos>(door)
        .ok()
        .ok_or_else(|| format!("{performer_name} lost track of the door."))?;
    if Ok(door_pos.get_area()) != world.get::<&Pos>(performer).map(|pos| pos.get_area()) {
        return Err(format!("{performer_name} cannot reach the door from here."));
    }

    let movement =
        position::prepare_move(world, performer, door_pos).map_err(Blockage::into_message)?;
    context.capture_frame_for_dialogue();
    movement.perform(context.mut_world()).unwrap();
    if assisting {
        context.add_dialogue(performer, "I'll help you get that door open.");
    }
    let world = context.mut_world();

    let door_pair = world
        .get::<&Door>(door)
        .map_err(|_| "The door ceased being a door.".to_string())?
        .door_pair;

    let block_type = *world
        .get::<&BlockType>(door_pair)
        .map_err(|_| "The door does not seem to be stuck.".to_string())?;

    match check_tool_for_forcing(block_type, world, performer, &performer_name) {
        Err(message) => {
            on_door_failure(context.state, performer, door, block_type);
            Err(message)
        }
        Ok(tool) => {
            world.remove_one::<BlockType>(door_pair).unwrap();
            if tool == Tool::Blowtorch {
                let doors = world
                    .query::<&Door>()
                    .iter()
                    .filter(|&(_, door)| door.door_pair == door_pair)
                    .map(|(entity, _)| entity)
                    .collect::<Vec<_>>();
                for door in doors {
                    world.insert_one(door, IsCut).unwrap();
                }
            }

            action::ok(tool.into_message(&performer_name))
        }
    }
}

fn on_door_failure(state: &mut GameState, performer: Entity, door: Entity, block_type: BlockType) {
    let world = &mut state.world;
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if !core::is_safe(world, area) {
        return;
    }

    let crew_member = world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .find(|&(crew_member, pos)| {
            crew_member != state.controlled
                && pos.is_in(area)
                && block_type
                    .usable_tools()
                    .into_iter()
                    .any(|tool| inventory::is_holding_tool(world, crew_member, tool))
        })
        .map(|(crew_member, _)| crew_member);
    if let Some(crew_member) = crew_member {
        world
            .insert_one(crew_member, Intention::Force(door))
            .unwrap();
    }
}

pub(super) fn go_to_ship(mut context: Context, performer: Entity) -> action::Result {
    let world = context.mut_world();
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if area::is_ship(area, world) {
        let _ = world.remove_one::<GoingToShip>(performer);
        return action::silent_ok();
    }

    let path = find_path_towards(world, area, |area| area::is_ship(area, world))
        .ok_or_else(|| "Could not find a path to the ship.".to_string())?;

    let result = enter_door(context.state, performer, path);

    let world = context.mut_world();
    let area = world.get::<&Pos>(performer).unwrap().get_area();
    if result.is_ok() && core::is_safe(world, area) && !area::is_ship(area, world) {
        world.insert_one(performer, GoingToShip).unwrap();
    } else {
        let _ = world.remove_one::<GoingToShip>(performer);
    }
    result
}

struct PathSearchEntry {
    path: Entity,
    area: Entity,
}

impl PathSearchEntry {
    fn start(path_entity: Entity, path: &Door) -> Self {
        Self {
            path: path_entity,
            area: path.destination.get_area(),
        }
    }

    fn next(&self, path: &Door) -> Self {
        Self {
            path: self.path,
            area: path.destination.get_area(),
        }
    }
}

fn find_path_towards(
    world: &World,
    area: Entity,
    predicate: impl Fn(Entity) -> bool,
) -> Option<Entity> {
    let mut entries = world
        .query::<(&Pos, &Door)>()
        .iter()
        .filter(|&(_, (pos, _))| pos.is_in(area))
        .map(|(entity, (_, path))| PathSearchEntry::start(entity, path))
        .collect::<Vec<_>>();
    let mut checked_areas = HashSet::from([area]);

    while !entries.is_empty() {
        let mut new_entries = vec![];
        for entry in entries {
            if checked_areas.insert(entry.area) {
                if predicate(entry.area) {
                    return Some(entry.path);
                }
                new_entries.extend(
                    world
                        .query::<(&Pos, &Door)>()
                        .iter()
                        .filter(|&(_, (pos, _))| pos.is_in(entry.area))
                        .map(|(_, (_, path))| entry.next(path)),
                );
            }
        }
        entries = new_entries;
    }

    None
}
