use crate::action::door::BlockType;
use crate::action::FortunaChest;
use crate::area::door::{place_pair, DoorInfo, DoorType};
use crate::area::{creature, door, Area, BackgroundType};
use crate::core::item;
use crate::core::position::{Coord, Pos};
use crate::status::Stats;
use crate::view::{AftikColor, DisplayInfo, NameData, TextureType};
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
            background: None,
            background_offset: None,
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
    background: Option<BackgroundType>,
    background_offset: Option<Coord>,
    objects: Vec<String>,
    symbols: HashMap<char, SymbolData>,
}

impl AreaData {
    pub fn door_symbol(
        &mut self,
        symbol: char,
        display_type: DoorType,
        adjective: Option<door::Adjective>,
        pair_id: &str,
    ) -> &mut Self {
        self.symbols.insert(
            symbol,
            SymbolData::Door {
                pair_id: pair_id.to_string(),
                display_type,
                adjective,
            },
        );
        self
    }

    fn build(self, builder: &mut Builder) -> Result<(), String> {
        let room = builder.world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
            background: self.background,
            background_offset: self.background_offset,
        },));

        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, builder.world);
            for symbol in objects.chars() {
                match self.symbols.get(&symbol) {
                    Some(SymbolData::Door {
                        pair_id,
                        display_type,
                        adjective,
                    }) => place_door(builder, pos, pair_id, symbol, *display_type, *adjective)?,
                    Some(SymbolData::Shopkeeper { items, color }) => {
                        creature::place_shopkeeper(builder.world, pos, items, *color)?
                    }
                    Some(SymbolData::Recruitable { name, stats, color }) => {
                        creature::place_recruitable(
                            builder.world,
                            pos,
                            name,
                            stats.clone(),
                            *color,
                        );
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
        adjective: Option<door::Adjective>,
    },
    Shopkeeper {
        items: Vec<item::Type>,
        color: AftikColor,
    },
    Recruitable {
        name: String,
        stats: Stats,
        color: AftikColor,
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
    symbol: char,
    display_type: DoorType,
    adjective: Option<door::Adjective>,
) -> Result<(), String> {
    let status = builder
        .doors
        .get_mut(pair_id)
        .ok_or_else(|| format!("Unknown door id \"{}\"", pair_id))?;
    let door_info = DoorInfo {
        pos,
        symbol,
        texture_type: display_type.into(),
        kind: display_type.into(),
        name: display_type.name_data(adjective),
    };

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
        DoorStatus::Placed => {
            return Err(format!("Doors for \"{}\" placed more than twice", pair_id))
        }
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
        'f' => item::Type::FuelCan.spawn(builder.world, pos),
        'c' => item::Type::Crowbar.spawn(builder.world, pos),
        'b' => item::Type::Blowtorch.spawn(builder.world, pos),
        'k' => item::Type::Keycard.spawn(builder.world, pos),
        'K' => item::Type::Knife.spawn(builder.world, pos),
        'B' => item::Type::Bat.spawn(builder.world, pos),
        's' => item::Type::Sword.spawn(builder.world, pos),
        'm' => item::Type::MeteorChunk.spawn(builder.world, pos),
        'a' => item::Type::AncientCoin.spawn(builder.world, pos),
        'G' => creature::place_goblin(builder.world, pos),
        'E' => creature::place_eyesaur(builder.world, pos),
        'Z' => creature::place_azureclops(builder.world, pos),
        '¤' => place_fortuna_chest(builder.world, pos),
        _ => return Err(format!("Unknown symbol \"{}\"", symbol)),
    }
    Ok(())
}

fn place_fortuna_chest(world: &mut World, pos: Pos) {
    world.spawn((
        DisplayInfo::new('¤', TextureType::FortunaChest, 20),
        NameData::from_noun("fortuna chest", "fortuna chests"),
        pos,
        FortunaChest,
    ));
}
