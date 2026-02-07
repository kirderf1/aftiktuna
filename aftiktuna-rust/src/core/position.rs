use super::area::Area;
use super::behavior::Hostile;
use super::name::NameData;
use crate::asset::GameAssets;
use hecs::{Entity, EntityRef, NoSuchEntity, World};
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, max, min};
use std::ops::RangeBounds;

use super::CrewMember;

pub type Coord = u32;

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
        self.try_offset(direction.into(), world)
    }

    pub fn try_offset(self, offset: i32, world: &World) -> Option<Pos> {
        let coord = self.coord.checked_add_signed(offset)?;
        Pos::try_new(self.area, coord, world).ok()
    }

    pub fn is_in(&self, area: Entity) -> bool {
        self.get_area().eq(&area)
    }

    pub fn distance_to(&self, pos: Pos) -> u32 {
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
    pub fn between(from: Pos, to: Pos) -> Self {
        from.assert_is_in_same_area(to);
        Self::between_coords(from.get_coord(), to.get_coord())
    }

    pub fn between_coords(from: Coord, to: Coord) -> Self {
        if to < from { Self::Left } else { Self::Right }
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

impl From<Direction> for i16 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
}

impl From<Direction> for i32 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
}

impl From<Direction> for f32 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Left => -1.,
            Direction::Right => 1.,
        }
    }
}

/// A creature with this component is treated as if they also occupy the space directly behind them.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Large;

pub type PlacementQuery<'a> = (&'a Pos, Option<&'a Direction>, hecs::Satisfies<&'a Large>);

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub pos: Pos,
    pub direction: Direction,
    pub is_large: bool,
}

impl From<(&Pos, Option<&Direction>, bool)> for Placement {
    fn from((pos, direction, is_large): (&Pos, Option<&Direction>, bool)) -> Self {
        Self {
            pos: *pos,
            direction: direction.copied().unwrap_or_default(),
            is_large,
        }
    }
}

impl Placement {
    pub fn area(&self) -> Entity {
        self.pos.get_area()
    }

    fn min_coord(&self) -> Coord {
        let coord = self.pos.get_coord();
        if self.is_large && self.direction == Direction::Right {
            self.pos.get_coord().checked_add_signed(-1).unwrap_or(coord)
        } else {
            self.pos.get_coord()
        }
    }

    fn max_coord(&self) -> Coord {
        let coord = self.pos.get_coord();
        if self.is_large && self.direction == Direction::Left {
            self.pos.get_coord().checked_add_signed(1).unwrap_or(coord)
        } else {
            self.pos.get_coord()
        }
    }

    pub fn closest_pos(&self, pos: Pos) -> Pos {
        pos.assert_is_in_same_area(self.pos);
        if self.is_large {
            let area = self.pos.get_area();
            let min_coord = self.min_coord();
            if pos.get_coord() <= min_coord {
                Pos {
                    coord: min_coord,
                    area,
                }
            } else {
                Pos {
                    coord: self.max_coord(),
                    area,
                }
            }
        } else {
            self.pos
        }
    }

    pub fn distance_to(&self, pos: Pos) -> u32 {
        self.closest_pos(pos).distance_to(pos)
    }

    pub fn get_adjacent_towards(&self, pos: Pos) -> Pos {
        self.closest_pos(pos).get_adjacent_towards(pos)
    }

    fn overlaps_with_pos(&self, pos: Pos) -> bool {
        self.overlaps_with_range(pos.area, &(pos.coord..=pos.coord))
    }

    fn overlaps_with_range(&self, area: Entity, range: &impl RangeBounds<Coord>) -> bool {
        let coord = self.pos.get_coord();

        self.pos.is_in(area)
            && (range.contains(&coord)
                || (self.is_large
                    && coord
                        .checked_add_signed(self.direction.opposite().into())
                        .is_some_and(|coord| range.contains(&coord))))
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

pub(crate) fn move_adjacent_placement(
    world: &mut World,
    entity: Entity,
    target_placement: Placement,
    assets: &GameAssets,
) -> Result<(), String> {
    let movement = prepare_move_adjacent_placement(world, entity, target_placement)
        .map_err(|blockage| blockage.into_message(world, assets))?;
    movement.perform(world).unwrap();
    Ok(())
}

pub(crate) fn prepare_move(
    world: &World,
    entity: Entity,
    destination: Pos,
) -> Result<Movement, Blockage> {
    let entity_ref = world.entity(entity).unwrap();
    let position = *entity_ref.get::<&Pos>().unwrap();
    position.assert_is_in_same_area(destination);

    check_is_blocked(world, entity_ref, position, destination)?;

    Ok(Movement::init(entity, position, destination))
}

pub(crate) fn prepare_move_adjacent_placement(
    world: &World,
    entity: Entity,
    target_placement: Placement,
) -> Result<Movement, Blockage> {
    let entity_ref = world.entity(entity).unwrap();
    let position = *entity_ref.get::<&Pos>().unwrap();
    position.assert_is_in_same_area(target_placement.pos);
    let target_pos = target_placement.closest_pos(position);
    let move_target = target_pos.get_adjacent_towards(position);

    check_is_blocked(world, entity_ref, position, move_target)?;

    Ok(if position != target_pos {
        let direction = Direction::between(position, target_pos);
        Movement::some(entity, move_target, direction)
    } else {
        Movement::none(entity)
    })
}

pub(crate) fn push_and_move(
    world: &mut World,
    entity: Entity,
    destination: Pos,
    assets: &GameAssets,
) -> Result<(), String> {
    let entity_ref = world.entity(entity).unwrap();
    let position = *entity_ref.get::<&Pos>().unwrap();
    position.assert_is_in_same_area(destination);

    if let Err(blockage) = check_is_blocked(world, entity_ref, position, destination) {
        blockage
            .try_push(Direction::between(position, destination), world)
            .map_err(|_| blockage.into_message(world, assets))?;
    }

    Movement::init(entity, position, destination)
        .perform(world)
        .unwrap();
    Ok(())
}

/// Expects entity to have both a Pos and a Direction.
pub(crate) fn turn_towards(world: &World, entity: Entity, target_pos: Pos) {
    let placement = Placement::from(
        world
            .query_one::<PlacementQuery>(entity)
            .unwrap()
            .get()
            .unwrap(),
    );
    let new_direction = Direction::between(placement.pos, target_pos);
    if placement.direction != new_direction {
        *world.get::<&mut Direction>(entity).unwrap() = new_direction;
        if placement.is_large {
            let new_pos = placement
                .pos
                .try_offset_direction(new_direction, world)
                .unwrap();
            *world.get::<&mut Pos>(entity).unwrap() = new_pos;
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OccupiesSpace {
    pub(crate) blocks_opponent: bool,
}

impl Default for OccupiesSpace {
    fn default() -> Self {
        Self {
            blocks_opponent: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Blockage {
    Hostile(Entity),
    TakesSpace([Entity; 2]),
}

impl Blockage {
    pub fn into_message(self, world: &World, assets: &GameAssets) -> String {
        match self {
            Blockage::Hostile(entity) => {
                format!(
                    "{} is in the way.",
                    NameData::find(world, entity, assets).definite(),
                )
            }
            Blockage::TakesSpace([entity1, entity2]) => {
                format!(
                    "{} and {} are in the way.",
                    NameData::find(world, entity1, assets).definite(),
                    NameData::find(world, entity2, assets).definite(),
                )
            }
        }
    }

    pub fn try_push(self, direction: Direction, world: &mut World) -> Result<(), PushError> {
        let Blockage::TakesSpace(entities) = self else {
            return Err(PushError);
        };
        let entity = entities
            .into_iter()
            .find(|&entity| world.satisfies::<&CrewMember>(entity).unwrap_or(false))
            .ok_or(PushError)?;
        let pos = world
            .get::<&Pos>(entity)
            .as_deref()
            .copied()
            .map_err(|_| PushError)?;
        for direction in [direction, direction.opposite()] {
            let Some(offset_pos) = pos.try_offset_direction(direction, world) else {
                continue;
            };
            if let Ok(movement) = prepare_move(world, entity, offset_pos) {
                movement.perform(world).unwrap();
                return Ok(());
            }
        }
        Err(PushError)
    }
}

pub struct PushError;

pub(crate) fn check_is_blocked(
    world: &World,
    entity_ref: EntityRef,
    entity_pos: Pos,
    target_pos: Pos,
) -> Result<(), Blockage> {
    if entity_pos.get_coord() == target_pos.get_coord() {
        return Ok(());
    }

    let adjacent_pos = entity_pos.get_adjacent_towards(target_pos);
    let min = min(adjacent_pos.get_coord(), target_pos.get_coord());
    let max = max(adjacent_pos.get_coord(), target_pos.get_coord());
    if entity_ref.has::<CrewMember>()
        && let Some(entity) =
            find_blocking_in_range::<&Hostile>(world, entity_pos.get_area(), min..=max)
    {
        return Err(Blockage::Hostile(entity));
    }
    if entity_ref.has::<Hostile>()
        && let Some(entity) =
            find_blocking_in_range::<&CrewMember>(world, entity_pos.get_area(), min..=max)
    {
        return Err(Blockage::Hostile(entity));
    }

    check_is_pos_blocked(Some(entity_ref.entity()), target_pos, world)
}

pub(crate) fn check_is_pos_blocked(
    ignored: Option<Entity>,
    target_pos: Pos,
    world: &World,
) -> Result<(), Blockage> {
    let entities_at_target = world
        .query::<PlacementQuery>()
        .with::<&OccupiesSpace>()
        .iter()
        .filter(|&(entity, query)| {
            Some(entity) != ignored && Placement::from(query).overlaps_with_pos(target_pos)
        })
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

fn find_blocking_in_range<Q: hecs::Query>(
    world: &World,
    area: Entity,
    range: impl RangeBounds<Coord>,
) -> Option<Entity> {
    world
        .query::<(PlacementQuery, &OccupiesSpace)>()
        .with::<Q>()
        .iter()
        .find(|&(_, (query, occupies_space))| {
            occupies_space.blocks_opponent
                && Placement::from(query).overlaps_with_range(area, &range)
        })
        .map(|(entity, _)| entity)
}
