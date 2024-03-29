use std::ops::Deref;

mod action;
mod command;
mod core;
pub mod game_interface;
pub mod location;
pub mod macroquad_interface;
pub mod serialization;
pub mod standard_io_interface;
pub mod view;

fn deref_clone<T: Clone>(value: impl Deref<Target = T>) -> T {
    value.deref().clone()
}
