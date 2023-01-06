use crate::action::combat::{IsFoe, Target, Weapon};
use crate::action::{combat, item, Action, CrewMember};
use crate::position::Pos;
use crate::status;
use crate::view::DisplayInfo;
use hecs::{Entity, World};

pub fn tick(world: &mut World) {
    let foes = world
        .query::<()>()
        .with::<&IsFoe>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for foe in foes {
        foe_ai(world, foe);
    }

    let aftiks = world
        .query::<()>()
        .with::<&CrewMember>()
        .iter()
        .map(|(entity, ())| entity)
        .collect::<Vec<_>>();
    for aftik in aftiks {
        aftik_ai(world, aftik);
    }
}

fn foe_ai(world: &mut World, foe: Entity) {
    if status::is_alive(foe, world) && world.get::<&Action>(foe).is_err() {
        if let Some(action) = pick_foe_action(world, foe) {
            world.insert_one(foe, action).unwrap();
        }
    }
}

fn pick_foe_action(world: &World, foe: Entity) -> Option<Action> {
    let pos = *world.get::<&Pos>(foe).ok()?;
    if world
        .query::<&Pos>()
        .with::<&CrewMember>()
        .iter()
        .any(|(_, aftik_pos)| aftik_pos.is_in(pos.get_area()))
    {
        Some(Action::AttackNearest(Target::Aftik))
    } else {
        None
    }
}

fn aftik_ai(world: &mut World, aftik: Entity) {
    if status::is_alive(aftik, world) && world.get::<&Action>(aftik).is_err() {
        if let Some(action) = pick_aftik_action(world, aftik) {
            world.insert_one(aftik, action).unwrap();
        }
    }
}

fn pick_aftik_action(world: &World, aftik: Entity) -> Option<Action> {
    let pos = *world.get::<&Pos>(aftik).ok()?;
    if world
        .query::<&Pos>()
        .with::<&IsFoe>()
        .iter()
        .any(|(_, foe_pos)| foe_pos.is_in(pos.get_area()))
    {
        return Some(Action::AttackNearest(Target::Foe));
    }

    let weapon_damage = combat::get_weapon_damage(world, aftik);

    for item in item::get_inventory(world, aftik) {
        if let Ok(weapon) = world.get::<&Weapon>(item) {
            if weapon.0 > weapon_damage {
                return Some(Action::Wield(
                    item,
                    DisplayInfo::find_definite_name(world, item),
                ));
            }
        }
    }

    None
}
