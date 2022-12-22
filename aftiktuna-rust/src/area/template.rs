use crate::area::{creature, item, Area};
use crate::position::Pos;
use hecs::{Entity, World};

pub struct Room {
    name: String,
    objects: Vec<String>,
}

impl Room {
    pub fn create(name: &str, objects: &[&str]) -> Room {
        Room {
            name: name.to_string(),
            objects: objects.iter().map(ToString::to_string).collect(),
        }
    }

    pub fn build(self, world: &mut World) -> Entity {
        let room = world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
        },));

        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, world);
            for symbol in objects.chars() {
                place_object(world, pos, symbol);
            }
        }

        room
    }
}

fn place_object(world: &mut World, pos: Pos, symbol: char) {
    match symbol {
        'f' => item::place_fuel(world, pos),
        'c' => item::place_crowbar(world, pos),
        'b' => item::place_blowtorch(world, pos),
        'k' => item::place_keycard(world, pos),
        'K' => item::place_knife(world, pos),
        'B' => item::place_bat(world, pos),
        's' => item::place_sword(world, pos),
        'G' => creature::place_goblin(world, pos),
        'E' => creature::place_eyesaur(world, pos),
        'Z' => creature::place_azureclops(world, pos),
        _ => panic!("Unknown symbol: {}", symbol),
    }
}
