use super::LocationGenContext;
use crate::core::area::{Area, BackgroundId};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::{Coord, Direction, Pos};
use crate::core::{item, FortunaChest};
use hecs::World;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::fs::File;

pub mod creature;
pub mod door;

mod container {
    use super::loot::LootTableId;
    use super::Builder;
    use crate::core::display::{ModelId, OrderWeight, Symbol};
    use crate::core::inventory::{Container, Held};
    use crate::core::item;
    use crate::core::name::Noun;
    use crate::core::position::{Direction, Pos};
    use hecs::Entity;
    use serde::{Deserialize, Serialize};

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
        Loot { table: LootTableId },
    }

    impl ItemOrLoot {
        fn generate(&self, container: Entity, builder: &mut Builder) -> Result<(), String> {
            let item = match self {
                ItemOrLoot::Item { item } => *item,
                ItemOrLoot::Loot { table } => builder
                    .loot_table_cache
                    .get_or_load(table)?
                    .pick_loot_item(&mut builder.gen_context.rng),
            };
            item.spawn(
                &mut builder.gen_context.world,
                Held::in_inventory(container),
            );
            Ok(())
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ContainerData {
        container_type: ContainerType,
        content: Vec<ItemOrLoot>,
        #[serde(default)]
        direction: Direction,
    }

    impl ContainerData {
        pub fn place(&self, pos: Pos, symbol: Symbol, builder: &mut Builder) -> Result<(), String> {
            let container = builder.gen_context.world.spawn((
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
}

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
    door_pairs: HashMap<String, door::DoorPairData>,
}

impl LocationData {
    pub fn load_from_json(name: &str) -> Result<Self, String> {
        crate::load_json_simple(format!("location/{name}.json"))
    }

    pub fn build(self, gen_context: &mut LocationGenContext) -> Result<Pos, String> {
        let mut builder = Builder::new(gen_context, self.door_pairs);
        let base_symbols = builtin_symbols()?;

        for area in self.areas {
            area.build(&mut builder, &base_symbols)?;
        }

        builder.door_pair_builder.verify_all_doors_placed()?;

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
        builder: &mut Builder,
        parent_symbols: &HashMap<char, SymbolData>,
    ) -> Result<(), String> {
        let room = builder.gen_context.world.spawn((Area {
            size: self.objects.len(),
            label: self.name,
            background: self.background,
            background_offset: self.background_offset,
        },));

        let symbols = Symbols::new(parent_symbols, &self.symbols);

        for (coord, objects) in self.objects.iter().enumerate() {
            let pos = Pos::new(room, coord, &builder.gen_context.world);
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
    Door(door::DoorSpawnData),
    Inanimate {
        model: ModelId,
        #[serde(default)]
        direction: Direction,
    },
    Container(container::ContainerData),
    Creature(creature::CreatureSpawnData),
    Shopkeeper(creature::ShopkeeperSpawnData),
    Character(creature::NpcSpawnData),
    AftikCorpse(creature::AftikCorpseData),
}

impl SymbolData {
    fn place(&self, pos: Pos, symbol: Symbol, builder: &mut Builder) -> Result<(), String> {
        match self {
            SymbolData::LocationEntry => builder.add_entry_pos(pos),
            SymbolData::FortunaChest => {
                place_fortuna_chest(&mut builder.gen_context.world, symbol, pos)
            }
            SymbolData::Item { item } => {
                item.spawn(&mut builder.gen_context.world, pos);
            }
            SymbolData::Loot { table } => {
                let item = builder
                    .loot_table_cache
                    .get_or_load(table)?
                    .pick_loot_item(&mut builder.gen_context.rng);
                item.spawn(&mut builder.gen_context.world, pos);
            }
            SymbolData::Door(door_data) => door_data.place(pos, symbol, builder)?,
            SymbolData::Inanimate { model, direction } => {
                builder.gen_context.world.spawn((
                    symbol,
                    model.clone(),
                    OrderWeight::Background,
                    pos,
                    *direction,
                ));
            }
            SymbolData::Container(container_data) => container_data.place(pos, symbol, builder)?,
            SymbolData::Creature(creature_data) => {
                creature_data.place(pos, symbol, builder.gen_context)
            }
            SymbolData::Shopkeeper(shopkeeper_data) => {
                shopkeeper_data.place(pos, &mut builder.gen_context.world)?
            }
            SymbolData::Character(npc_data) => npc_data.place(pos, builder.gen_context),
            SymbolData::AftikCorpse(aftik_corpse_data) => {
                aftik_corpse_data.place(pos, builder.gen_context)
            }
        }
        Ok(())
    }
}

struct Builder<'a> {
    gen_context: &'a mut LocationGenContext,
    entry_positions: Vec<Pos>,
    door_pair_builder: door::DoorPairsBuilder,
    loot_table_cache: loot::LootTableCache,
}

impl<'a> Builder<'a> {
    fn new(
        gen_context: &'a mut LocationGenContext,
        door_pair_data: HashMap<String, door::DoorPairData>,
    ) -> Self {
        Self {
            gen_context,
            entry_positions: Vec::new(),
            door_pair_builder: door::DoorPairsBuilder::init(door_pair_data),
            loot_table_cache: loot::LootTableCache::default(),
        }
    }

    fn get_random_entry_pos(&mut self) -> Result<Pos, String> {
        self.entry_positions
            .choose(&mut self.gen_context.rng)
            .copied()
            .ok_or_else(|| "No entry point was set!".to_string())
    }

    fn add_entry_pos(&mut self, pos: Pos) {
        self.entry_positions.push(pos);
    }
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
