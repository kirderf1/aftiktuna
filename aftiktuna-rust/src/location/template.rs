use super::creature::{self, AftikProfile};
use super::door::{self, place_pair, DoorInfo, DoorType};
use super::{Area, BackgroundId};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::inventory::{Container, Held};
use crate::core::name::Noun;
use crate::core::position::{Coord, Direction, Pos};
use crate::core::{item, BlockType, FortunaChest};
use hecs::{Entity, World};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::fs::File;

mod loot {
    use crate::core::item;
    use rand::distributions::WeightedIndex;
    use rand::Rng;
    use serde::{Deserialize, Serialize};
    use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

    #[derive(Debug, Deserialize)]
    struct LootEntry {
        item: item::Type,
        weight: u16,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct LootTableId(String);

    pub struct LootTable {
        entries: Vec<LootEntry>,
        index_distribution: WeightedIndex<u16>,
    }

    impl LootTable {
        fn load(LootTableId(name): &LootTableId) -> Result<Self, String> {
            let entries: Vec<LootEntry> =
                crate::load_json_simple(format!("loot_table/{name}.json"))?;
            let index_distribution = WeightedIndex::new(entries.iter().map(|entry| entry.weight))
                .map_err(|error| error.to_string())?;
            Ok(Self {
                entries,
                index_distribution,
            })
        }

        pub fn pick_loot_item(&self, rng: &mut impl Rng) -> item::Type {
            self.entries[rng.sample(&self.index_distribution)].item
        }
    }

    #[derive(Default)]
    pub struct LootTableCache(HashMap<LootTableId, LootTable>);

    impl LootTableCache {
        pub fn get_or_load(&mut self, loot_table_id: &LootTableId) -> Result<&LootTable, String> {
            match self.0.entry(loot_table_id.clone()) {
                HashMapEntry::Occupied(entry) => Ok(entry.into_mut()),
                HashMapEntry::Vacant(entry) => {
                    let loot_table = LootTable::load(loot_table_id)?;
                    Ok(entry.insert(loot_table))
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    areas: Vec<AreaData>,
    door_pairs: HashMap<String, DoorPairData>,
}

impl LocationData {
    pub fn load_from_json(name: &str) -> Result<Self, String> {
        crate::load_json_simple(format!("location/{name}.json"))
    }

    pub fn build(
        self,
        world: &mut World,
        character_profiles: &mut Vec<AftikProfile>,
        rng: &mut impl Rng,
    ) -> Result<Pos, String> {
        let mut builder = Builder::new(world, character_profiles, rng, &self.door_pairs);
        let base_symbols = builtin_symbols()?;

        for area in self.areas {
            area.build(&mut builder, &base_symbols)?;
        }

        verify_placed_doors(&builder)?;

        builder.get_random_entry_pos()
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
        builder: &mut Builder<impl Rng>,
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
                    Some(symbol_data) => symbol_data.place(pos, Symbol(symbol), builder)?,
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
        table: loot::LootTableId,
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
    Container(ContainerData),
    Creature(creature::CreatureSpawnData),
    Shopkeeper(creature::ShopkeeperSpawnData),
    Character(creature::NpcSpawnData),
    AftikCorpse(creature::AftikCorpseData),
}

impl SymbolData {
    fn place(
        &self,
        pos: Pos,
        symbol: Symbol,
        builder: &mut Builder<impl Rng>,
    ) -> Result<(), String> {
        match self {
            SymbolData::LocationEntry => builder.add_entry_pos(pos),
            SymbolData::FortunaChest => place_fortuna_chest(builder.world, symbol, pos),
            SymbolData::Item { item } => {
                item.spawn(builder.world, pos);
            }
            SymbolData::Loot { table } => {
                let item = builder
                    .loot_table_cache
                    .get_or_load(table)?
                    .pick_loot_item(builder.rng);
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
            SymbolData::Container(container_data) => {
                container_data.place(pos, symbol, builder)?;
            }
            SymbolData::Creature(creature_data) => {
                creature_data.place(pos, symbol, builder.world, builder.rng)
            }
            SymbolData::Shopkeeper(shopkeeper_data) => shopkeeper_data.place(pos, builder.world)?,
            SymbolData::Character(npc_data) => {
                npc_data.place(pos, builder.world, builder.character_profiles, builder.rng)
            }
            SymbolData::AftikCorpse(aftik_corpse_data) => {
                aftik_corpse_data.place(pos, builder.world, builder.character_profiles, builder.rng)
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

struct Builder<'a, R: Rng> {
    world: &'a mut World,
    character_profiles: &'a mut Vec<AftikProfile>,
    rng: &'a mut R,
    entry_positions: Vec<Pos>,
    doors: HashMap<String, DoorStatus<'a>>,
    loot_table_cache: loot::LootTableCache,
}

impl<'a, R: Rng> Builder<'a, R> {
    fn new(
        world: &'a mut World,
        character_profiles: &'a mut Vec<AftikProfile>,
        rng: &'a mut R,
        door_pairs: &'a HashMap<String, DoorPairData>,
    ) -> Self {
        Self {
            world,
            character_profiles,
            rng,
            entry_positions: Vec::new(),
            doors: door_pairs
                .iter()
                .map(|(key, data)| (key.to_string(), DoorStatus::None(data)))
                .collect(),
            loot_table_cache: loot::LootTableCache::default(),
        }
    }

    fn get_random_entry_pos(&mut self) -> Result<Pos, String> {
        self.entry_positions
            .choose(self.rng)
            .copied()
            .ok_or_else(|| "No entry point was set!".to_string())
    }

    fn add_entry_pos(&mut self, pos: Pos) {
        self.entry_positions.push(pos);
    }
}

enum DoorStatus<'a> {
    None(&'a DoorPairData),
    One(&'a DoorPairData, DoorInfo),
    Placed,
}

fn place_door(
    builder: &mut Builder<impl Rng>,
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

fn verify_placed_doors(builder: &Builder<impl Rng>) -> Result<(), String> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ContainerType {
    Tent,
    Cabinet,
    Drawer,
    Crate,
    Chest,
}

impl ContainerType {
    fn model_id(self) -> ModelId {
        ModelId::new(match self {
            ContainerType::Tent => "tent",
            ContainerType::Cabinet => "cabinet",
            ContainerType::Drawer => "drawer",
            ContainerType::Crate => "crate",
            ContainerType::Chest => "chest",
        })
    }

    fn noun(self) -> Noun {
        match self {
            ContainerType::Tent => Noun::new("tent", "tents"),
            ContainerType::Cabinet => Noun::new("cabinet", "cabinets"),
            ContainerType::Drawer => Noun::new("drawer", "drawers"),
            ContainerType::Crate => Noun::new("crate", "crates"),
            ContainerType::Chest => Noun::new("chest", "chests"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ItemOrLoot {
    Item { item: item::Type },
    Loot { table: loot::LootTableId },
}

impl ItemOrLoot {
    fn generate(&self, container: Entity, builder: &mut Builder<impl Rng>) -> Result<(), String> {
        let item = match self {
            ItemOrLoot::Item { item } => *item,
            ItemOrLoot::Loot { table } => builder
                .loot_table_cache
                .get_or_load(table)?
                .pick_loot_item(builder.rng),
        };
        item.spawn(builder.world, Held::in_inventory(container));
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContainerData {
    container_type: ContainerType,
    content: Vec<ItemOrLoot>,
    #[serde(default)]
    direction: Direction,
}

impl ContainerData {
    fn place(
        &self,
        pos: Pos,
        symbol: Symbol,
        builder: &mut Builder<impl Rng>,
    ) -> Result<(), String> {
        let container = builder.world.spawn((
            symbol,
            self.container_type.model_id(),
            OrderWeight::Background,
            self.container_type.noun(),
            pos,
            self.direction,
            Container,
        ));
        for entry in &self.content {
            entry.generate(container, builder)?;
        }
        Ok(())
    }
}
