use crate::core::position::{Coord, Pos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Area {
    pub size: Coord,
    pub label: String,
    pub background: BackgroundType,
    pub background_offset: Option<Coord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BackgroundType(String);

impl BackgroundType {
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

impl Default for BackgroundType {
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
