use crate::core::position::{Coord, Pos};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Area {
    pub size: Coord,
    pub label: String,
    pub background: BackgroundId,
    pub background_offset: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BackgroundId(pub String);

impl BackgroundId {
    pub fn blank() -> Self {
        Self::new("blank")
    }
    pub fn location_choice() -> Self {
        Self::new("location_choice")
    }
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl Default for BackgroundId {
    fn default() -> Self {
        Self::blank()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ship {
    pub status: ShipStatus,
    pub exit_pos: Pos,
    pub item_pos: Pos,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FuelAmount {
    OneCan,
    TwoCans,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShipStatus {
    NeedFuel(FuelAmount),
    Refueled,
    Launching,
}

#[derive(Serialize, Deserialize)]
pub struct ShipControls;

pub fn is_ship(area: Entity, world: &World) -> bool {
    world.satisfies::<&Ship>(area).unwrap_or(false)
}
