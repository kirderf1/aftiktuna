use std::fmt::Display;
use std::fs::File;
use std::ops::Deref;

use serde::de::DeserializeOwned;

mod action;
mod ai;
pub mod command;
pub mod core;
pub mod game_interface;
pub mod game_loop;
pub mod location;
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

enum OneOrTwo<T> {
    One(T),
    Two(T, T),
}

fn try_combine_adjacent<T>(items: Vec<T>, combine: impl Fn(T, T) -> OneOrTwo<T>) -> Vec<T> {
    let mut output = Vec::new();
    let mut iter = items.into_iter();
    let Some(mut prev_item) = iter.next() else {
        return output;
    };
    for next_item in iter {
        match combine(prev_item, next_item) {
            OneOrTwo::One(item) => prev_item = item,
            OneOrTwo::Two(item_1, item_2) => {
                output.push(item_1);
                prev_item = item_2;
            }
        }
    }
    output.push(prev_item);
    output
}
