use crate::core::position::{Coord, Pos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Area {
    pub size: Coord,
    pub label: String,
    pub background: BackgroundType,
    pub background_offset: Option<Coord>,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundType {
    #[default]
    Blank,
    LocationChoice,
    Ship,
    ForestEntrance,
    Forest,
    Field,
    Shack,
    FacilityOutside,
    FacilitySize3,
    FacilitySize4,
    FacilitySize5,
    FacilitySize6,
    FacilitySize7,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ship {
    pub status: ShipStatus,
    pub exit_pos: Pos,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShipStatus {
    NeedTwoCans,
    NeedOneCan,
    Refueled,
    Launching,
}

#[derive(Serialize, Deserialize)]
pub struct ShipControls;
