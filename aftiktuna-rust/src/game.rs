use specs::{prelude::*, Component};

pub use position::{Coord, Position};
use specs::storage::MaskedStorage;
use std::ops::Deref;
use view::{GOType, Messages};

use crate::GameState;

mod position;
pub mod view;

const AREA_SIZE: Coord = 5;

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct FuelCan;

pub fn init_area(world: &mut World) -> Entity {
    let aftik = world
        .create_entity()
        .with(GOType::new('A', "Aftik"))
        .with(Position::new(1))
        .build();
    place_fuel(world, 4);
    place_fuel(world, 4);
    aftik
}

fn place_fuel(world: &mut World, pos: Coord) {
    world
        .create_entity()
        .with(GOType::new('f', "Fuel can"))
        .with(Position::new(pos))
        .with(FuelCan)
        .build();
}

pub struct TakeFuelCan;

impl<'a> System<'a> for TakeFuelCan {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, FuelCan>,
        WriteExpect<'a, GameState>,
        WriteExpect<'a, Messages>,
    );

    fn run(
        &mut self,
        (entities, mut pos, fuel_markers, mut game_state, mut messages): Self::SystemData,
    ) {
        let optional = find_fuel_can(&entities, &pos, &fuel_markers);

        match optional {
            Some((fuel_can, item_pos)) => {
                let aftik = game_state.aftik.expect("Expected aftik to be initialized");
                pos.get_mut(aftik).unwrap().move_to(item_pos);
                entities.delete(fuel_can).unwrap();
                game_state.has_won = true;

                messages.0.push("You picked up the fuel can.".to_string());
            }
            None => {
                messages
                    .0
                    .push("There is no fuel can here to pick up.".to_string());
            }
        }
    }
}

pub fn find_fuel_can<'a, P>(
    entities: &Entities,
    pos: &Storage<'a, Position, P>, //Any kind of position storage, could be either a WriteStorage<> or a ReadStorage<>
    fuel_markers: &ReadStorage<FuelCan>,
) -> Option<(Entity, Coord)>
where
    P: Deref<Target = MaskedStorage<Position>>,
{
    // Return any entity with the "fuel can" marker
    (entities, pos, fuel_markers)
        .join()
        .next()
        .map(|pair| (pair.0, pair.1.get_coord()))
}
