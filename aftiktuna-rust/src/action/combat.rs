use crate::position::{try_move_aftik, Position};
use crate::view::DisplayInfo;
use hecs::{Entity, World};

#[derive(Debug)]
pub struct IsFoe;

#[derive(Debug)]
pub struct Health(pub i32);

pub fn attack(world: &mut World, aftik: Entity, target: Entity) -> Result<String, String> {
    let name = world.get::<DisplayInfo>(target).unwrap().name().to_string();
    let target_pos = world.get::<Position>(target).unwrap().0;
    let aftik_pos = world.get::<Position>(aftik).unwrap().0;

    try_move_aftik(world, aftik, target_pos.get_adjacent_towards(aftik_pos))?;

    let killed = hit(world, target, 1);

    if killed {
        world.despawn(target).unwrap();
        Ok(format!("You attacked and killed the {}.", name))
    } else {
        Ok(format!("You attacked the {}.", name))
    }
}

pub fn hit(world: &mut World, target: Entity, damage: i32) -> bool {
    if let Ok(mut health) = world.get_mut::<Health>(target) {
        health.0 -= damage;
        health.0 <= 0
    } else {
        false
    }
}
