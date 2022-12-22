use crate::area::{creature, item, Area};
use crate::position::Pos;
use hecs::{Entity, World};

pub struct AreaData {
    name: String,
    objects: Vec<String>,
}

impl AreaData {
    pub fn create(name: &str, objects: &[&str]) -> AreaData {
        AreaData {
            name: name.to_string(),
            objects: objects.iter().map(ToString::to_string).collect(),
        }
    }

    pub fn build(self, builder: &mut Builder) -> Entity {
        let room = builder.world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
        },));

        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, builder.world);
            for symbol in objects.chars() {
                place_object(builder, pos, symbol);
            }
        }

        room
    }
}

pub struct Builder<'a> {
    world: &'a mut World,
    entry: Option<Pos>,
}

impl<'a> Builder<'a> {
    pub fn new(world: &mut World) -> Builder {
        Builder { world, entry: None }
    }

    pub fn get_entry(&self) -> Pos {
        match self.entry {
            None => panic!("No entry point was set!"),
            Some(pos) => pos,
        }
    }

    fn set_entry(&mut self, pos: Pos) {
        if self.entry.is_some() {
            panic!("Entry has already been set!");
        } else {
            self.entry = Some(pos);
        }
    }
}

fn place_object(builder: &mut Builder, pos: Pos, symbol: char) {
    match symbol {
        'v' => builder.set_entry(pos),
        'f' => item::place_fuel(builder.world, pos),
        'c' => item::place_crowbar(builder.world, pos),
        'b' => item::place_blowtorch(builder.world, pos),
        'k' => item::place_keycard(builder.world, pos),
        'K' => item::place_knife(builder.world, pos),
        'B' => item::place_bat(builder.world, pos),
        's' => item::place_sword(builder.world, pos),
        'G' => creature::place_goblin(builder.world, pos),
        'E' => creature::place_eyesaur(builder.world, pos),
        'Z' => creature::place_azureclops(builder.world, pos),
        _ => panic!("Unknown symbol: {}", symbol),
    }
}
