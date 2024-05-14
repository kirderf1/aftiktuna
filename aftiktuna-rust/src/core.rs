use crate::location::LocationTracker;
use crate::view::StatusCache;
use crate::{location, serialization};
use hecs::{Entity, World};
use position::Pos;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

pub mod area;
pub mod inventory;
pub mod item;
pub mod position;
pub mod status;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrewMember(pub Entity);

#[derive(Debug, Serialize, Deserialize)]
pub struct IsFoe;

#[derive(Serialize, Deserialize)]
pub struct Waiting;

#[derive(Serialize, Deserialize)]
pub struct Recruitable;

#[derive(Serialize, Deserialize)]
pub struct FortunaChest;

#[derive(Serialize, Deserialize)]
pub struct OpenedChest;

#[derive(Serialize, Deserialize)]
pub struct Points(pub i32);

#[derive(Serialize, Deserialize)]
pub struct Shopkeeper(pub Vec<PricedItem>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricedItem {
    pub item: item::Type,
    pub price: i32,
}

#[derive(Serialize, Deserialize)]
pub struct IsTrading(pub Entity);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Door {
    pub kind: DoorKind,
    pub destination: Pos,
    pub door_pair: Entity,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DoorKind {
    Door,
    Path,
}

#[derive(Serialize, Deserialize)]
pub struct IsCut;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Stuck,
    Sealed,
    Locked,
}

impl BlockType {
    pub fn description(self) -> &'static str {
        match self {
            BlockType::Stuck => "stuck",
            BlockType::Sealed => "sealed shut",
            BlockType::Locked => "locked",
        }
    }

    pub fn usable_tools(self) -> Vec<item::Tool> {
        match self {
            BlockType::Stuck => vec![item::Tool::Crowbar, item::Tool::Blowtorch],
            BlockType::Sealed | BlockType::Locked => vec![item::Tool::Blowtorch],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GoingToShip;

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
        .with::<&IsFoe>()
        .iter()
        .all(|(_, pos)| !pos.is_in(area))
}

pub fn get_wielded_weapon_modifier(world: &World, attacker: Entity) -> f32 {
    inventory::get_wielded(world, attacker)
        .and_then(|item| world.get::<&item::Weapon>(item).map(|weapon| weapon.0).ok())
        .unwrap_or(2.0)
}
