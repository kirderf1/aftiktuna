use std::fmt::Display;
use std::fs::File;
use std::ops::Deref;

use serde::de::DeserializeOwned;

mod action;
mod ai;
mod command;
pub mod core;
pub mod game_interface;
mod game_loop;
pub mod location;
pub mod macroquad_interface;
pub mod serialization;
pub mod standard_io_interface;
pub mod view;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value.eq(&Default::default())
}

fn deref_clone<T: Clone>(value: impl Deref<Target = T>) -> T {
    value.deref().clone()
}

fn load_json_simple<T: DeserializeOwned>(path: impl Display) -> Result<T, String> {
    let file = File::open(format!("assets/{path}"))
        .map_err(|error| format!("Failed to open file: {error}"))?;
    serde_json::from_reader(file).map_err(|error| format!("Failed to parse file: {error}"))
}
