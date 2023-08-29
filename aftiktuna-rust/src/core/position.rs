use crate::action::CrewMember;
use crate::area::Area;
use crate::view::name::NameData;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min, Ordering};

pub type Coord = usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pos {
    coord: Coord,
    area: Entity,
}

impl Pos {
    pub fn new(area: Entity, coord: Coord, world: &World) -> Pos {
        assert_valid_coord(area, coord, world);
        Pos { coord, area }
    }

    pub fn center_of(area: Entity, world: &World) -> Pos {
        let size = world.get::<&Area>(area).unwrap().size;
        Pos::new(area, (size - 1) / 2, world)
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
        .get::<&Area>(area)
        .expect("Expected given area to have an area component")
        .size;
    assert!(
        coord < area_size,
        "Position {} is out of bounds for room with size {}.",
        coord,
        area_size
    );
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MovementBlocking;

pub fn try_move(world: &mut World, entity: Entity, destination: Pos) -> Result<(), String> {
    let position = *world.get::<&Pos>(entity).unwrap();
    assert_eq!(
        destination.get_area(),
        position.get_area(),
        "Areas should be equal when called."
    );

    if position == destination {
        return Ok(());
    }

    check_is_blocked(world, entity, position, destination).map_err(Blockage::into_message)?;

    world
        .insert(
            entity,
            (destination, Direction::between(position, destination)),
        )
        .unwrap();
    Ok(())
}

pub fn try_move_adjacent(world: &mut World, entity: Entity, target: Pos) -> Result<(), String> {
    let position = *world.get::<&Pos>(entity).unwrap();
    let move_target = target.get_adjacent_towards(position);

    if position != move_target {
        try_move(world, entity, move_target)
    } else {
        set_direction_towards(world, entity, target);
        Ok(())
    }
}

pub struct Blockage(Entity, NameData);

impl Blockage {
    pub fn into_message(self) -> String {
        format!("{} is in the way.", self.1.definite())
    }
}

pub fn check_is_blocked(
    world: &World,
    entity: Entity,
    entity_pos: Pos,
    target_pos: Pos,
) -> Result<(), Blockage> {
    if world.get::<&CrewMember>(entity).is_err() {
        return Ok(()); //Only aftiks are blocked.
    }

    if entity_pos.get_coord() == target_pos.get_coord() {
        return Ok(());
    }

    let adjacent_pos = entity_pos.get_adjacent_towards(target_pos);
    let min = min(adjacent_pos.get_coord(), target_pos.get_coord());
    let max = max(adjacent_pos.get_coord(), target_pos.get_coord());
    if let Some((entity, _)) = world
        .query::<&Pos>()
        .with::<&MovementBlocking>()
        .iter()
        .find(|(_, pos)| {
            pos.is_in(entity_pos.get_area()) && min <= pos.get_coord() && pos.get_coord() <= max
        })
    {
        Err(Blockage(entity, NameData::find(world, entity)))
    } else {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    pub fn between(from: Pos, to: Pos) -> Direction {
        if to.get_coord() < from.get_coord() {
            Direction::Left
        } else {
            Direction::Right
        }
    }

    pub fn towards_center(pos: Pos, world: &World) -> Direction {
        let center = Pos::center_of(pos.get_area(), world);
        Direction::between(pos, center)
    }
}

pub fn set_direction_towards(world: &mut World, entity: Entity, target: Pos) {
    let position = *world.get::<&Pos>(entity).unwrap();

    if position != target {
        world
            .insert_one(entity, Direction::between(position, target))
            .unwrap();
    }
}
