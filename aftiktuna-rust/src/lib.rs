use serde::{Deserialize, Serialize};
use std::ops::Deref;
mod action;
mod ai;
pub mod asset;
mod command;
pub mod core;
pub mod game_interface;
mod game_loop;
pub mod location;
pub mod serialization;
pub mod standard_io_interface;
pub mod view;

pub use command::CommandInfo;
pub use command::suggestion as command_suggestion;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum StopType {
    Win,
    Lose,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value.eq(&Default::default())
}

fn deref_clone<T: Clone>(value: impl Deref<Target = T>) -> T {
    value.deref().clone()
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
