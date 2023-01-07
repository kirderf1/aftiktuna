use crate::action::door::BlockType;
use crate::area::creature::{place_shopkeeper, ShopItem};
use crate::area::door::{place_pair, DoorInfo, DoorType};
use crate::area::{creature, door, Area};
use crate::item;
use crate::position::Pos;
use hecs::World;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
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

    pub fn area(&mut self, name: &str, objects: &[&str]) -> &mut AreaData {
        self.areas.push(AreaData {
            name: name.to_string(),
            objects: objects.iter().map(ToString::to_string).collect(),
            symbols: HashMap::new(),
        });
        self.areas.last_mut().unwrap() //Should not be None since we just added to the vec
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

    pub fn build(self, world: &mut World) -> Result<Pos, String> {
        let mut builder = Builder::new(world, &self.door_pairs);

        for area in self.areas {
            area.build(&mut builder)?;
        }

        verify_placed_doors(&builder)?;

        builder.get_entry()
    }
}

#[derive(Serialize, Deserialize)]
pub struct AreaData {
    name: String,
    objects: Vec<String>,
    symbols: HashMap<char, SymbolData>,
}

impl AreaData {
    pub fn door_symbol(
        &mut self,
        symbol: char,
        display_type: DoorType,
        pair_id: &str,
    ) -> &mut Self {
        self.symbols.insert(
            symbol,
            SymbolData::Door {
                pair_id: pair_id.to_string(),
                display_type,
            },
        );
        self
    }

    fn build(self, builder: &mut Builder) -> Result<(), String> {
        let room = builder.world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
        },));

        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, builder.world);
            for symbol in objects.chars() {
                match self.symbols.get(&symbol) {
                    Some(SymbolData::Door {
                        pair_id,
                        display_type,
                    }) => place_door(builder, pos, pair_id, display_type)?,
                    Some(SymbolData::Shopkeeper { item }) => {
                        place_shopkeeper(builder.world, pos, *item)
                    }
                    None => place_object(builder, pos, symbol)?,
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SymbolData {
    Door {
        pair_id: String,
        display_type: DoorType,
    },
    Shopkeeper {
        item: ShopItem,
    },
}

#[derive(Serialize, Deserialize)]
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

    fn get_entry(&self) -> Result<Pos, String> {
        self.entry
            .ok_or_else(|| "No entry point was set!".to_string())
    }

    fn set_entry(&mut self, pos: Pos) -> Result<(), String> {
        if self.entry.is_some() {
            Err("Entry has already been set!".to_string())
        } else {
            self.entry = Some(pos);
            Ok(())
        }
    }
}

enum DoorStatus<'a> {
    None(&'a DoorPairData),
    One(&'a DoorPairData, DoorInfo),
    Placed,
}

fn place_door(
    builder: &mut Builder,
    pos: Pos,
    pair_id: &str,
    display_type: &DoorType,
) -> Result<(), String> {
    let status = builder
        .doors
        .get_mut(pair_id)
        .ok_or_else(|| format!("Unknown door id: {}", pair_id))?;
    let door_info = DoorInfo(pos, door::door_display(display_type));

    *status = match status {
        DoorStatus::None(data) => DoorStatus::One(data, door_info),
        DoorStatus::One(data, other_door) => {
            place_pair(
                builder.world,
                door_info,
                other_door.clone(),
                data.block_type,
            );
            DoorStatus::Placed
        }
        DoorStatus::Placed => return Err(format!("Door placed more than twice: {}", pair_id)),
    };
    Ok(())
}

fn verify_placed_doors(builder: &Builder) -> Result<(), String> {
    for (pair_id, status) in &builder.doors {
        match status {
            DoorStatus::Placed => {}
            _ => return Err(format!("Door pair was not fully placed: {}", pair_id)),
        }
    }
    Ok(())
}

fn place_object(builder: &mut Builder, pos: Pos, symbol: char) -> Result<(), String> {
    match symbol {
        'v' => builder.set_entry(pos)?,
        'f' => {
            item::spawn_fuel_can(builder.world, pos);
        }
        'c' => {
            item::spawn_crowbar(builder.world, pos);
        }
        'b' => {
            item::spawn_blowtorch(builder.world, pos);
        }
        'k' => {
            item::spawn_keycard(builder.world, pos);
        }
        'K' => {
            item::spawn_knife(builder.world, pos);
        }
        'B' => {
            item::spawn_bat(builder.world, pos);
        }
        's' => {
            item::spawn_sword(builder.world, pos);
        }
        'm' => {
            item::spawn_meteor_chunk(builder.world, pos);
        }
        'a' => {
            item::spawn_ancient_coin(builder.world, pos);
        }
        'G' => creature::place_goblin(builder.world, pos),
        'E' => creature::place_eyesaur(builder.world, pos),
        'Z' => creature::place_azureclops(builder.world, pos),
        _ => return Err(format!("Unknown symbol: {}", symbol)),
    }
    Ok(())
}
