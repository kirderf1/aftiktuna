use crate::area::Area;
use crate::position::Pos;
use hecs::{Entity, World};

pub struct Room {
    pub name: String,
    pub objects: Vec<String>,
}

impl Room {
    pub fn create(name: &str, objects: Vec<&str>) -> Room {
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
        _ => panic!("Unknown symbol: {}", symbol),
    }
}
