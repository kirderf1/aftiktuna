use crate::location::LocationTracker;
use crate::view::StatusCache;
use crate::{location, serialization};
use hecs::{Entity, World};
use position::Pos;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

pub mod ai;
pub mod area;
pub mod inventory;
pub mod item;
pub mod position;
pub mod status;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Serialize, Deserialize)]
pub struct Waiting;

#[derive(Serialize, Deserialize)]
pub struct Recruitable;

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
pub struct OpenedChest;

#[derive(Serialize, Deserialize)]
pub struct GameState {
    #[serde(with = "serialization::world")]
    pub world: World,
    #[serde(skip)]
    pub rng: ThreadRng,
    pub locations: LocationTracker,
    pub ship: Entity,
    pub controlled: Entity,
    pub status_cache: StatusCache,
    pub has_introduced_controlled: bool,
}

pub fn setup(locations: LocationTracker) -> GameState {
    let mut world = World::new();

    let (controlled, ship) = location::init(&mut world);

    GameState {
        world,
        rng: thread_rng(),
        locations,
        ship,
        controlled,
        status_cache: StatusCache::default(),
        has_introduced_controlled: false,
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum StopType {
    Win,
    Lose,
}

pub fn is_safe(world: &World, area: Entity) -> bool {
    world
        .query::<&Pos>()
        .with::<&ai::IsFoe>()
        .iter()
        .all(|(_, pos)| !pos.is_in(area))
}
