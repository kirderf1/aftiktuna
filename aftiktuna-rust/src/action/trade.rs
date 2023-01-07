use crate::action::item::InInventory;
use crate::action::CrewMember;
use crate::position::Pos;
use crate::view::DisplayInfo;
use crate::{item, position};
use hecs::{Entity, World};

pub struct Points(pub i32);

pub struct Shopkeeper;

pub fn trade(world: &mut World, performer: Entity, shopkeeper: Entity) -> Result<String, String> {
    let performer_name = DisplayInfo::find_definite_name(world, performer);
    let crew = world.get::<&CrewMember>(performer).unwrap().0;
    let performer_pos = *world.get::<&Pos>(performer).unwrap();
    let shop_pos = *world
        .get::<&Pos>(shopkeeper)
        .map_err(|_| format!("{} lost track of the door.", performer_name))?;
    world.get::<&Shopkeeper>(shopkeeper).unwrap();

    position::try_move(
        world,
        performer,
        shop_pos.get_adjacent_towards(performer_pos),
    )?;

    try_spend_points(world, crew, 3500)?;

    item::spawn_fuel_can(world, InInventory(performer));

    Ok(format!("{} bought 1 fuel can.", performer_name))
}

fn try_spend_points(world: &mut World, crew: Entity, points: i32) -> Result<(), String> {
    if let Ok(mut crew_points) = world.get::<&mut Points>(crew) {
        if crew_points.0 >= points {
            crew_points.0 -= points;
            Ok(())
        } else {
            Err("The crew cannot afford that.".to_string())
        }
    } else {
        Err("The crew is missing its wallet.".to_string())
    }
}
