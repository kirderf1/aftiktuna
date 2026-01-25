mod action;
mod ai;
pub mod asset;
mod command;
pub mod core;
mod dialogue;
pub mod game_interface;
mod game_loop;
pub mod location;
pub mod serialization;
pub mod view;

use serde::{Deserialize, Serialize};
use std::ops::{Add, Deref, Mul, Sub};

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

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum OneOrTwo<T> {
    One(T),
    Two(T, T),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum OneOrList<T> {
    One(T),
    List(Vec<T>),
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

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(
    from = "OneOrTwo<T>",
    into = "OneOrTwo<T>",
    bound = "for <'a> T: Copy + PartialEq + Serialize + Deserialize<'a>"
)]
pub struct Range<T>(pub T, pub T);

impl<T: Copy + Add<Output = T> + Sub<Output = T>> Range<T>
where
    f32: Mul<T, Output = T>,
{
    pub fn interpolate(&self, fraction: f32) -> T {
        self.0 + fraction * (self.1 - self.0)
    }
}

impl<T: Copy> From<OneOrTwo<T>> for Range<T> {
    fn from(value: OneOrTwo<T>) -> Self {
        match value {
            OneOrTwo::One(value) => Self(value, value),
            OneOrTwo::Two(value1, value2) => Self(value1, value2),
        }
    }
}

impl<T: PartialEq> From<Range<T>> for OneOrTwo<T> {
    fn from(value: Range<T>) -> Self {
        if value.0 == value.1 {
            Self::One(value.0)
        } else {
            Self::Two(value.0, value.1)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}
