use crate::action::door::BlockType;
use crate::area;
use crate::area::{creature, item, place_doors, Area, DoorInfo};
use crate::position::Pos;
use hecs::{Entity, World};
use std::collections::HashMap;

pub struct LocationData {
    areas: Vec<AreaData>,
    door_pairs: HashMap<String, DoorPairData>,
}

impl LocationData {
    pub fn new() -> Self {
        LocationData {
            areas: Vec::new(),
            door_pairs: HashMap::new(),
        }
    }

    pub fn area(&mut self, name: &str, objects: &[&str], doors: &[(char, &str)]) {
        self.areas.push(AreaData {
            name: name.to_string(),
            objects: objects.iter().map(ToString::to_string).collect(),
            doors: doors.iter().map(|(c, str)| (*c, str.to_string())).collect(),
        });
    }

    pub fn door(&mut self, key: &str) {
        self.door_pairs
            .insert(key.to_string(), DoorPairData { block_type: None });
    }
    pub fn blocked_door(&mut self, key: &str, block_type: BlockType) {
        self.door_pairs.insert(
            key.to_string(),
            DoorPairData {
                block_type: Some(block_type),
            },
        );
    }

    pub fn build(self, world: &mut World) -> Pos {
        let mut builder = Builder::new(world, &self.door_pairs);

        for area in self.areas {
            area.build(&mut builder);
        }

        verify_placed_doors(&builder);

        builder.get_entry()
    }
}

struct AreaData {
    name: String,
    objects: Vec<String>,
    doors: HashMap<char, String>,
}

impl AreaData {
    fn build(self, builder: &mut Builder) -> Entity {
        let room = builder.world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
        },));

        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, builder.world);
            for symbol in objects.chars() {
                match self.doors.get(&symbol) {
                    Some(pair_id) => place_door(builder, door_info(pos, symbol), pair_id),
                    None => place_object(builder, pos, symbol),
                }
            }
        }

        room
    }
}

struct DoorPairData {
    block_type: Option<BlockType>,
}

struct Builder<'a> {
    world: &'a mut World,
    entry: Option<Pos>,
    doors: HashMap<String, DoorStatus<'a>>,
}

impl<'a> Builder<'a> {
    fn new(world: &'a mut World, door_pairs: &'a HashMap<String, DoorPairData>) -> Self {
        Builder {
            world,
            entry: None,
            doors: door_pairs
                .iter()
                .map(|(key, data)| (key.to_string(), DoorStatus::None(data)))
                .collect(),
        }
    }

    fn get_entry(&self) -> Pos {
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

enum DoorStatus<'a> {
    None(&'a DoorPairData),
    One(&'a DoorPairData, DoorInfo),
    Placed,
}

fn door_info(pos: Pos, symbol: char) -> DoorInfo {
    let display_info = match symbol {
        '^' => area::door(),
        '<' => area::left_door(),
        '>' => area::right_door(),
        _ => panic!("Unknown door symbol: {}", symbol),
    };
    DoorInfo(pos, display_info)
}

fn place_door(builder: &mut Builder, door_info: DoorInfo, pair_id: &String) {
    let status = builder
        .doors
        .get_mut(pair_id)
        .expect(&format!("Unknown door id: {}", pair_id));
    *status = match status {
        DoorStatus::None(data) => DoorStatus::One(data, door_info),
        DoorStatus::One(data, other_door) => {
            place_doors(
                builder.world,
                door_info,
                other_door.clone(),
                data.block_type,
            );
            DoorStatus::Placed
        }
        DoorStatus::Placed => panic!("Door placed more than twice: {}", pair_id),
    }
}

fn verify_placed_doors(builder: &Builder) {
    for (pair_id, status) in builder.doors.iter() {
        match status {
            DoorStatus::Placed => {}
            _ => panic!("Door pair was not fully placed: {}", pair_id),
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
