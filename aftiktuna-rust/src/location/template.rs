use super::creature::{AftikProfile, ProfileOrRandom};
use super::door::{place_pair, DoorInfo, DoorType};
use super::{creature, door, Area, BackgroundId};
use crate::core::name::Noun;
use crate::core::position::{Coord, Direction, Pos};
use crate::core::{item, AftikColorId, BlockType, FortunaChest, ModelId, OrderWeight, Symbol};
use hecs::World;
use rand::distributions::WeightedIndex;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Entry as HashMapEntry, HashMap};
use std::fs::File;

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    areas: Vec<AreaData>,
    door_pairs: HashMap<String, DoorPairData>,
}

impl LocationData {
    pub fn build(
        self,
        world: &mut World,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) -> Result<Pos, String> {
        let mut builder = Builder::new(world, &self.door_pairs);
        let base_symbols = builtin_symbols()?;

        for area in self.areas {
            area.build(&mut builder, &base_symbols, character_profiles, rng)?;
        }

        verify_placed_doors(&builder)?;

        builder.get_random_entry_pos(rng)
    }
}

#[derive(Serialize, Deserialize)]
struct AreaData {
    name: String,
    #[serde(default)]
    background: BackgroundId,
    background_offset: Option<Coord>,
    objects: Vec<String>,
    symbols: HashMap<char, SymbolData>,
}

impl AreaData {
    fn build(
        self,
        builder: &mut Builder,
        parent_symbols: &HashMap<char, SymbolData>,
        character_profiles: &mut Vec<AftikProfile>,
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
                    Some(symbol_data) => {
                        symbol_data.place(pos, Symbol(symbol), builder, character_profiles, rng)?
                    }
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
        table: LootTableId,
    },
    Door {
        pair_id: String,
        display_type: DoorType,
        #[serde(default)]
        adjective: Option<door::Adjective>,
    },
    Inanimate {
        model: ModelId,
        #[serde(default)]
        direction: Direction,
    },
    Creature(creature::CreatureSpawnData),
    Shopkeeper {
        stock: Vec<creature::StockDefinition>,
        color: AftikColorId,
        #[serde(default)]
        direction: Option<Direction>,
    },
    Recruitable {
        #[serde(default)]
        profile: ProfileOrRandom,
        #[serde(default)]
        direction: Option<Direction>,
    },
    AftikCorpse {
        #[serde(default)]
        color: Option<AftikColorId>,
        #[serde(default)]
        direction: Option<Direction>,
    },
}

impl SymbolData {
    fn place(
        &self,
        pos: Pos,
        symbol: Symbol,
        builder: &mut Builder,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) -> Result<(), String> {
        match self {
            SymbolData::LocationEntry => builder.add_entry_pos(pos),
            SymbolData::FortunaChest => place_fortuna_chest(builder.world, symbol, pos),
            SymbolData::Item { item } => item.spawn(builder.world, pos),
            SymbolData::Loot { table } => {
                let item = builder.loot_table(table)?.pick_loot_item(rng);
                item.spawn(builder.world, pos);
            }
            SymbolData::Door {
                pair_id,
                display_type,
                adjective,
            } => place_door(builder, pos, pair_id, symbol, *display_type, *adjective)?,
            SymbolData::Inanimate { model, direction } => {
                builder.world.spawn((
                    symbol,
                    model.clone(),
                    OrderWeight::Background,
                    pos,
                    *direction,
                ));
            }
            SymbolData::Creature(spawn_data) => spawn_data.spawn(builder.world, symbol, pos, rng),
            SymbolData::Shopkeeper {
                stock,
                color,
                direction,
            } => creature::place_shopkeeper(builder.world, pos, stock, color.clone(), *direction)?,
            SymbolData::Recruitable { profile, direction } => {
                if let Some(profile) = profile.clone().unwrap(character_profiles, rng) {
                    creature::place_recruitable(builder.world, pos, profile, *direction);
                }
            }
            SymbolData::AftikCorpse { color, direction } => {
                if let Some(color) = color.clone().or_else(|| {
                    creature::remove_random_profile(character_profiles, rng).map(AftikColorId::from)
                }) {
                    creature::place_aftik_corpse(builder.world, pos, color, *direction);
                }
            }
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
    entry_positions: Vec<Pos>,
    doors: HashMap<String, DoorStatus<'a>>,
    loaded_loot_tables: HashMap<LootTableId, LootTable>,
}

impl<'a> Builder<'a> {
    fn new(world: &'a mut World, door_pairs: &'a HashMap<String, DoorPairData>) -> Self {
        Builder {
            world,
            entry_positions: Vec::new(),
            doors: door_pairs
                .iter()
                .map(|(key, data)| (key.to_string(), DoorStatus::None(data)))
                .collect(),
            loaded_loot_tables: HashMap::new(),
        }
    }

    fn get_random_entry_pos(&self, rng: &mut impl Rng) -> Result<Pos, String> {
        self.entry_positions
            .choose(rng)
            .copied()
            .ok_or_else(|| "No entry point was set!".to_string())
    }

    fn add_entry_pos(&mut self, pos: Pos) {
        self.entry_positions.push(pos);
    }

    fn loot_table(&mut self, loot_table_id: &LootTableId) -> Result<&LootTable, String> {
        match self.loaded_loot_tables.entry(loot_table_id.clone()) {
            HashMapEntry::Occupied(entry) => Ok(entry.into_mut()),
            HashMapEntry::Vacant(entry) => {
                let loot_table = LootTable::load(loot_table_id)?;
                Ok(entry.insert(loot_table))
            }
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
        model_id: display_type.into(),
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
        ModelId::new("fortuna_chest"),
        OrderWeight::Background,
        Noun::new("fortuna chest", "fortuna chests"),
        pos,
        FortunaChest,
    ));
}

#[derive(Debug, Deserialize)]
struct LootEntry {
    item: item::Type,
    weight: u16,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct LootTableId(String);

struct LootTable {
    entries: Vec<LootEntry>,
    index_distribution: WeightedIndex<u16>,
}

impl LootTable {
    fn load(LootTableId(name): &LootTableId) -> Result<Self, String> {
        let entries: Vec<LootEntry> = crate::load_json_simple(format!("loot_table/{name}.json"))?;
        let index_distribution = WeightedIndex::new(entries.iter().map(|entry| entry.weight))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            entries,
            index_distribution,
        })
    }

    fn pick_loot_item(&self, rng: &mut impl Rng) -> item::Type {
        self.entries[rng.sample(&self.index_distribution)].item
    }
}
