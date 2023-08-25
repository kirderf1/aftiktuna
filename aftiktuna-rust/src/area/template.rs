use crate::action::door::BlockType;
use crate::action::FortunaChest;
use crate::area::door::{place_pair, DoorInfo, DoorType};
use crate::area::{creature, door, Area, BackgroundType};
use crate::core::item;
use crate::core::position::{Coord, Pos};
use crate::core::status::Stats;
use crate::view::{AftikColor, DisplayInfo, NameData, TextureType};
use hecs::World;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    areas: Vec<AreaData>,
    door_pairs: HashMap<String, DoorPairData>,
}

impl LocationData {
    pub fn build(self, world: &mut World) -> Result<Pos, String> {
        let mut builder = Builder::new(world, &self.door_pairs);
        let base_symbols = builtin_symbols()?;

        for area in self.areas {
            area.build(&mut builder, &base_symbols)?;
        }

        verify_placed_doors(&builder)?;

        builder.get_entry()
    }
}

#[derive(Serialize, Deserialize)]
struct AreaData {
    name: String,
    background: Option<BackgroundType>,
    background_offset: Option<Coord>,
    objects: Vec<String>,
    symbols: HashMap<char, SymbolData>,
}

impl AreaData {
    fn build(
        self,
        builder: &mut Builder,
        parent_symbols: &HashMap<char, SymbolData>,
    ) -> Result<(), String> {
        let room = builder.world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
            background: self.background,
            background_offset: self.background_offset,
        },));

        let symbols = Symbols::new(parent_symbols, &self.symbols);
        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, builder.world);
            for symbol in objects.chars() {
                match symbols.lookup(symbol) {
                    Some(symbol_data) => symbol_data.place(builder, pos, symbol)?,
                    None => Err(format!("Unknown symbol \"{}\"", symbol))?,
                }
            }
        }
        Ok(())
    }
}

struct Symbols<'a> {
    parent_map: &'a HashMap<char, SymbolData>,
    map: &'a HashMap<char, SymbolData>,
}

impl<'a> Symbols<'a> {
    fn new(parent_map: &'a HashMap<char, SymbolData>, map: &'a HashMap<char, SymbolData>) -> Self {
        Self { parent_map, map }
    }

    fn lookup(&self, symbol: char) -> Option<&'a SymbolData> {
        self.map
            .get(&symbol)
            .or_else(|| self.parent_map.get(&symbol))
    }
}

fn builtin_symbols() -> Result<HashMap<char, SymbolData>, String> {
    let file = File::open("assets/symbols.json")
        .map_err(|error| format!("Failed to open symbols file: {error}"))?;
    serde_json::from_reader::<_, HashMap<char, SymbolData>>(file)
        .map_err(|error| format!("Failed to parse symbols file: {error}"))
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SymbolData {
    LocationEntry,
    FortunaChest,
    Item {
        item: item::Type,
    },
    Door {
        pair_id: String,
        display_type: DoorType,
        adjective: Option<door::Adjective>,
    },
    Creature {
        creature: creature::Type,
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

impl SymbolData {
    fn place(&self, builder: &mut Builder, pos: Pos, symbol: char) -> Result<(), String> {
        match self {
            SymbolData::LocationEntry => builder.set_entry(pos)?,
            SymbolData::FortunaChest => place_fortuna_chest(builder.world, pos),
            SymbolData::Item { item } => item.spawn(builder.world, pos),
            SymbolData::Door {
                pair_id,
                display_type,
                adjective,
            } => place_door(builder, pos, pair_id, symbol, *display_type, *adjective)?,
            SymbolData::Creature { creature } => creature.spawn(builder.world, pos),
            SymbolData::Shopkeeper { items, color } => {
                creature::place_shopkeeper(builder.world, pos, items, *color)?
            }
            SymbolData::Recruitable { name, stats, color } => {
                creature::place_recruitable(builder.world, pos, name, stats.clone(), *color)
            }
        }
        Ok(())
    }
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

fn place_fortuna_chest(world: &mut World, pos: Pos) {
    world.spawn((
        DisplayInfo::new('¤', TextureType::FortunaChest, 20),
        NameData::from_noun("fortuna chest", "fortuna chests"),
        pos,
        FortunaChest,
    ));
}
