use crate::area::Area;
use hecs::{Entity, With, World};
use std::cmp::{max, min, Ordering};

pub type Coord = usize;

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    coord: Coord,
    area: Entity,
}

impl Pos {
    pub fn new(area: Entity, coord: Coord, world: &World) -> Pos {
        assert_valid_coord(area, coord, world);
        Pos { coord, area }
    }

    pub fn get_coord(&self) -> Coord {
        self.coord
    }

    pub fn get_area(&self) -> Entity {
        self.area
    }

    pub fn get_adjacent_towards(&self, pos: Pos) -> Pos {
        assert_eq!(
            self.get_area(),
            pos.get_area(),
            "Positions must be in the same area."
        );
        match self.get_coord().cmp(&pos.get_coord()) {
            Ordering::Less => Pos {
                coord: self.coord + 1,
                area: self.area,
            },
            Ordering::Greater => Pos {
                coord: self.coord - 1,
                area: self.area,
            },
            Ordering::Equal => *self,
        }
    }

    pub fn is_in(&self, area: Entity) -> bool {
        self.get_area().eq(&area)
    }

    pub fn distance_to(&self, pos: Pos) -> usize {
        if self.get_coord() > pos.get_coord() {
            self.get_coord() - pos.get_coord()
        } else {
            pos.get_coord() - self.get_coord()
        }
    }
}

fn assert_valid_coord(area: Entity, coord: Coord, world: &World) {
    let area_size = world
        .get::<Area>(area)
        .expect("Expected given area to have an area component")
        .size;
    assert!(
        coord < area_size,
        "Position {} is out of bounds for room with size {}.",
        coord,
        area_size
    );
}

#[derive(Debug, Default)]
pub struct MovementBlocking;

pub fn try_move_aftik(world: &mut World, aftik: Entity, pos: Pos) -> Result<(), String> {
    let aftik_pos = *world.get::<Pos>(aftik).unwrap();
    assert_eq!(
        pos.get_area(),
        aftik_pos.get_area(),
        "Areas should be equal when called."
    );

    if is_blocked_for_aftik(world, aftik_pos, pos) {
        Err("Something is in the way.".to_string())
    } else {
        world.insert_one(aftik, pos).unwrap();
        Ok(())
    }
}

pub fn is_blocked_for_aftik(world: &World, aftik_pos: Pos, target_pos: Pos) -> bool {
    if aftik_pos.get_coord() == target_pos.get_coord() {
        return false;
    }

    let adjacent_pos = aftik_pos.get_adjacent_towards(target_pos);
    let min = min(adjacent_pos.get_coord(), target_pos.get_coord());
    let max = max(adjacent_pos.get_coord(), target_pos.get_coord());
    world
        .query::<With<MovementBlocking, &Pos>>()
        .iter()
        .any(|(_, pos)| {
            pos.is_in(aftik_pos.get_area()) && min <= pos.get_coord() && pos.get_coord() <= max
        })
}
