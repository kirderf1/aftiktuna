use crate::action::door::BlockType;
use crate::action::FortunaChest;
use crate::core::item;
use crate::core::position::{Coord, Direction, Pos};
use crate::core::status::Stats;
use crate::location::door::{place_pair, DoorInfo, DoorType};
use crate::location::{creature, door, Area, BackgroundType};
use crate::view::area::{AftikColor, OrderWeight, Symbol, TextureType};
use crate::view::name::Noun;
use hecs::World;
use rand::distributions::WeightedIndex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    areas: Vec<AreaData>,
    door_pairs: HashMap<String, DoorPairData>,
}

impl LocationData {
    pub fn build(self, world: &mut World, rng: &mut impl Rng) -> Result<Pos, String> {
        let mut builder = Builder::new(world, &self.door_pairs);
        let base_symbols = builtin_symbols()?;

        for area in self.areas {
            area.build(&mut builder, &base_symbols, rng)?;
        }

        verify_placed_doors(&builder)?;

        builder.get_entry()
    }
}

#[derive(Serialize, Deserialize)]
struct AreaData {
    name: String,
    #[serde(default)]
    background: BackgroundType,
    background_offset: Option<Coord>,
    objects: Vec<String>,
    symbols: HashMap<char, SymbolData>,
}

impl AreaData {
    fn build(
        self,
        builder: &mut Builder,
        parent_symbols: &HashMap<char, SymbolData>,
        rng: &mut impl Rng,
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
                    Some(symbol_data) => symbol_data.place(builder, rng, pos, Symbol(symbol))?,
                    None => Err(format!("Unknown symbol \"{symbol}\""))?,
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
    Loot {
        table: LootTable,
    },
    Door {
        pair_id: String,
        display_type: DoorType,
        #[serde(default)]
        adjective: Option<door::Adjective>,
    },
    Creature {
        creature: creature::Type,
        #[serde(default)]
        direction: Option<Direction>,
    },
    Shopkeeper {
        items: Vec<item::Type>,
        color: AftikColor,
        #[serde(default)]
        direction: Option<Direction>,
    },
    Recruitable {
        name: String,
        stats: Stats,
        color: AftikColor,
        #[serde(default)]
        direction: Option<Direction>,
    },
}

impl SymbolData {
    fn place(
        &self,
        builder: &mut Builder,
        rng: &mut impl Rng,
        pos: Pos,
        symbol: Symbol,
    ) -> Result<(), String> {
        match self {
            SymbolData::LocationEntry => builder.set_entry(pos)?,
            SymbolData::FortunaChest => place_fortuna_chest(builder.world, symbol, pos),
            SymbolData::Item { item } => item.spawn(builder.world, pos),
            SymbolData::Loot { table } => {
                let item = table.random_loot(rng);
                item.spawn(builder.world, pos);
            }
            SymbolData::Door {
                pair_id,
                display_type,
                adjective,
            } => place_door(builder, pos, pair_id, symbol, *display_type, *adjective)?,
            SymbolData::Creature {
                creature,
                direction,
            } => creature.spawn(builder.world, symbol, pos, *direction),
            SymbolData::Shopkeeper {
                items,
                color,
                direction,
            } => creature::place_shopkeeper(builder.world, symbol, pos, items, *color, *direction)?,
            SymbolData::Recruitable {
                name,
                stats,
                color,
                direction,
            } => creature::place_recruitable(
                builder.world,
                symbol,
                pos,
                name,
                stats.clone(),
                *color,
                *direction,
            ),
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct DoorPairData {
    #[serde(default)]
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
    symbol: Symbol,
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
        name: display_type.noun(adjective),
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

fn place_fortuna_chest(world: &mut World, symbol: Symbol, pos: Pos) {
    world.spawn((
        symbol,
        TextureType::new("fortuna_chest"),
        OrderWeight::Background,
        Noun::new("fortuna chest", "fortuna chests"),
        pos,
        FortunaChest,
    ));
}

macro_rules! unzip {
    ($($item:expr, $weight:expr);* $(;)?) => {
        ([$($item),*], [$($weight),*])
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LootTable {
    Regular,
    Valuable,
}

impl LootTable {
    fn random_loot(self, rng: &mut impl Rng) -> item::Type {
        match self {
            LootTable::Regular => {
                let (items, weights) = unzip!(
                    item::Type::FoodRation, 20;
                    item::Type::Crowbar, 2;
                    item::Type::Knife, 7;
                    item::Type::Bat, 4;
                    item::Type::MeteorChunk, 2;
                    item::Type::AncientCoin, 10;
                );
                let index_distribution = WeightedIndex::new(weights).unwrap();
                items[rng.sample(index_distribution)]
            }
            LootTable::Valuable => {
                let (items, weights) = unzip!(
                    item::Type::Crowbar, 5;
                    item::Type::Blowtorch, 3;
                    item::Type::Bat, 5;
                    item::Type::Sword, 2;
                    item::Type::Medkit, 4;
                    item::Type::MeteorChunk, 8;
                );
                let index_distribution = WeightedIndex::new(weights).unwrap();
                items[rng.sample(index_distribution)]
            }
        }
    }
}
