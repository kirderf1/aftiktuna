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
        Pos::try_new(area, coord, world).unwrap()
    }

    pub fn try_new(area: Entity, coord: Coord, world: &World) -> Result<Pos, PosError> {
        let area_size = world
            .get::<&Area>(area)
            .map_err(|_| PosError::InvalidArea)?
            .size;
        if coord >= area_size {
            return Err(PosError::OutOfBounds { coord, area_size });
        }
        Ok(Pos { coord, area })
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

    pub fn assert_is_in_same_area(self, pos: Pos) {
        assert_eq!(
            self.get_area(),
            pos.get_area(),
            "These positions must be in the same area."
        )
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

    pub fn try_offset_direction(self, direction: Direction, world: &World) -> Option<Pos> {
        self.try_offset(
            match direction {
                Direction::Left => -1,
                Direction::Right => 1,
            },
            world,
        )
    }

    pub fn try_offset(self, offset: isize, world: &World) -> Option<Pos> {
        let coord = self.coord.checked_add_signed(offset)?;
        Pos::try_new(self.area, coord, world).ok()
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

#[derive(Debug)]
pub enum PosError {
    InvalidArea,
    OutOfBounds { coord: Coord, area_size: Coord },
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

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
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

    fn init(entity: Entity, from: Pos, to: Pos) -> Self {
        if from == to {
            Self::none(entity)
        } else {
            Self::some(entity, to, Direction::between(from, to))
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
    let entity_ref = world.entity(entity).unwrap();
    let position = *entity_ref.get::<&Pos>().unwrap();
    position.assert_is_in_same_area(destination);

    check_is_blocked(world, entity_ref, position, destination)?;

    Ok(Movement::init(entity, position, destination))
}

pub fn prepare_move_adjacent(
    world: &World,
    entity: Entity,
    target: Pos,
) -> Result<Movement, Blockage> {
    let entity_ref = world.entity(entity).unwrap();
    let position = *entity_ref.get::<&Pos>().unwrap();
    position.assert_is_in_same_area(target);
    let move_target = target.get_adjacent_towards(position);

    check_is_blocked(world, entity_ref, position, move_target)?;

    Ok(if position != target {
        let direction = Direction::between(position, target);
        Movement::some(entity, move_target, direction)
    } else {
        Movement::none(entity)
    })
}

pub fn push_and_move(world: &mut World, entity: Entity, destination: Pos) -> Result<(), String> {
    let entity_ref = world.entity(entity).unwrap();
    let position = *entity_ref.get::<&Pos>().unwrap();
    position.assert_is_in_same_area(destination);

    if let Err(blockage) = check_is_blocked(world, entity_ref, position, destination) {
        if blockage
            .try_push(Direction::between(position, destination), world)
            .is_err()
        {
            return Err(blockage.into_message(world));
        }
    }

    Movement::init(entity, position, destination)
        .perform(world)
        .unwrap();
    Ok(())
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct OccupiesSpace;

#[derive(Debug, Clone, Copy)]
pub enum Blockage {
    Hostile(Entity),
    TakesSpace([Entity; 2]),
}

impl Blockage {
    pub fn into_message(self, world: &World) -> String {
        match self {
            Blockage::Hostile(entity) => {
                format!(
                    "{} is in the way.",
                    NameData::find(world, entity).definite(),
                )
            }
            Blockage::TakesSpace([entity1, entity2]) => {
                format!(
                    "{} and {} are in the way.",
                    NameData::find(world, entity1).definite(),
                    NameData::find(world, entity2).definite(),
                )
            }
        }
    }

    fn try_push(self, direction: Direction, world: &mut World) -> Result<(), ()> {
        let Blockage::TakesSpace(entities) = self else {
            return Err(());
        };
        let entity = entities
            .into_iter()
            .find(|&entity| world.satisfies::<&CrewMember>(entity).unwrap_or(false))
            .ok_or(())?;
        let pos = world
            .get::<&Pos>(entity)
            .as_deref()
            .copied()
            .map_err(|_| ())?;
        for direction in [direction, direction.opposite()] {
            let Some(offset_pos) = pos.try_offset_direction(direction, world) else {
                continue;
            };
            if let Ok(movement) = prepare_move(world, entity, offset_pos) {
                movement.perform(world).unwrap();
                return Ok(());
            }
        }
        Err(())
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
            return Err(Blockage::Hostile(entity));
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
        return Err(Blockage::TakesSpace([
            entities_at_target[0],
            entities_at_target[1],
        ]));
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
