use super::name::NameData;
use super::{area::Area, Hostile};
use hecs::{Entity, EntityRef, NoSuchEntity, World};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min, Ordering};
use std::ops::RangeBounds;

use super::CrewMember;

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
        "Position {coord} is out of bounds for area with size {area_size}.",
    );
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    #[default]
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

pub struct Movement {
    entity: Entity,
    components: Option<(Pos, Direction)>,
}

impl Movement {
    pub fn perform(self, world: &mut World) -> Result<(), NoSuchEntity> {
        if let Some(components) = self.components {
            world.insert(self.entity, components)
        } else {
            Ok(())
        }
    }

    fn none(entity: Entity) -> Self {
        Self {
            entity,
            components: None,
        }
    }

    fn some(entity: Entity, pos: Pos, direction: Direction) -> Self {
        Self {
            entity,
            components: Some((pos, direction)),
        }
    }
}

pub fn move_to(world: &mut World, entity: Entity, destination: Pos) -> Result<(), String> {
    let movement = prepare_move(world, entity, destination)
        .map_err(|blockage| blockage.into_message(world))?;
    movement.perform(world).unwrap();
    Ok(())
}

pub fn move_adjacent(world: &mut World, entity: Entity, target: Pos) -> Result<(), String> {
    let movement = prepare_move_adjacent(world, entity, target)
        .map_err(|blockage| blockage.into_message(world))?;
    movement.perform(world).unwrap();
    Ok(())
}

pub fn prepare_move(world: &World, entity: Entity, destination: Pos) -> Result<Movement, Blockage> {
    let position = *world.get::<&Pos>(entity).unwrap();
    assert_eq!(
        position.get_area(),
        destination.get_area(),
        "Areas should be equal when called."
    );
    check_is_blocked(world, world.entity(entity).unwrap(), position, destination)?;

    Ok(if position == destination {
        Movement::none(entity)
    } else {
        Movement::some(
            entity,
            destination,
            Direction::between(position, destination),
        )
    })
}

pub fn prepare_move_adjacent(
    world: &World,
    entity: Entity,
    target: Pos,
) -> Result<Movement, Blockage> {
    let position = *world.get::<&Pos>(entity).unwrap();
    let move_target = target.get_adjacent_towards(position);

    assert_eq!(
        position.get_area(),
        move_target.get_area(),
        "Areas should be equal when called."
    );
    check_is_blocked(world, world.entity(entity).unwrap(), position, move_target)?;

    Ok(if position != target {
        let direction = Direction::between(position, target);
        Movement::some(entity, move_target, direction)
    } else {
        Movement::none(entity)
    })
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct OccupiesSpace;

pub struct Blockage(Entity);

impl Blockage {
    pub fn into_message(self, world: &World) -> String {
        format!(
            "{} is in the way.",
            NameData::find(world, self.0).definite()
        )
    }
}

pub fn check_is_blocked(
    world: &World,
    entity_ref: EntityRef,
    entity_pos: Pos,
    target_pos: Pos,
) -> Result<(), Blockage> {
    if entity_pos.get_coord() == target_pos.get_coord() {
        return Ok(());
    }
    if entity_ref.satisfies::<&CrewMember>() {
        let adjacent_pos = entity_pos.get_adjacent_towards(target_pos);
        let min = min(adjacent_pos.get_coord(), target_pos.get_coord());
        let max = max(adjacent_pos.get_coord(), target_pos.get_coord());
        if let Some(entity) =
            find_blocking_hostile_in_range(world, entity_pos.get_area(), min..=max)
        {
            return Err(Blockage(entity));
        }
    }

    let entities_at_target = world
        .query::<&Pos>()
        .with::<&OccupiesSpace>()
        .iter()
        .filter(|&(_, pos)| target_pos.eq(pos))
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if entities_at_target.len() >= 2 {
        return Err(Blockage(entities_at_target[0]));
    }

    Ok(())
}

fn find_blocking_hostile_in_range(
    world: &World,
    area: Entity,
    range: impl RangeBounds<usize>,
) -> Option<Entity> {
    world
        .query::<&Pos>()
        .with::<(&Hostile, &OccupiesSpace)>()
        .iter()
        .find(|(_, pos)| pos.is_in(area) && range.contains(&pos.get_coord()))
        .map(|(entity, _)| entity)
}
