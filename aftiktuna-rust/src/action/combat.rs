use crate::position::{try_move_aftik, Position};
use crate::view::DisplayInfo;
use hecs::{Entity, World};

#[derive(Debug)]
pub struct IsFoe;

pub fn attack(world: &mut World, aftik: Entity, target: Entity) -> Result<String, String> {
    let name = world.get::<DisplayInfo>(target).unwrap().name().to_string();
    let target_pos = world.get::<Position>(target).unwrap().0;
    let aftik_pos = world.get::<Position>(aftik).unwrap().0;

    try_move_aftik(world, aftik, target_pos.get_adjacent_towards(aftik_pos))?;

    world.despawn(target).unwrap();

    Ok(format!("You attacked and killed {}.", name))
}
