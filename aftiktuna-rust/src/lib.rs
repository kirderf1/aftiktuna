use std::ops::Deref;

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

fn deref_clone<T: Clone>(value: impl Deref<Target = T>) -> T {
    value.deref().clone()
}
