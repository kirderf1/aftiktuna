use super::LocationGenContext;
use crate::asset::{self, loot};
use crate::core::area::{Area, BackgroundId};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::name::Noun;
use crate::core::position::{Coord, Direction, Pos};
use crate::core::{item, FortunaChest};
use hecs::World;
use indexmap::IndexMap;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs::File;

pub mod creature;
pub mod door;

pub mod container {
    use super::Builder;
    use crate::asset::loot::LootTableId;
    use crate::core::display::{ModelId, OrderWeight, Symbol};
    use crate::core::inventory::{Container, Held};
    use crate::core::item;
    use crate::core::name::Noun;
    use crate::core::position::{Direction, Pos};
    use hecs::Entity;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ContainerType {
        Tent,
        Cabinet,
        Drawer,
        Crate,
        Chest,
    }

    impl ContainerType {
        pub fn variants() -> &'static [Self] {
            use ContainerType::*;
            &[Tent, Cabinet, Drawer, Crate, Chest]
        }

        pub fn model_id(self) -> ModelId {
            ModelId::new(match self {
                ContainerType::Tent => "tent",
                ContainerType::Cabinet => "cabinet",
                ContainerType::Drawer => "drawer",
                ContainerType::Crate => "crate",
                ContainerType::Chest => "chest",
            })
        }

        pub fn noun(self) -> Noun {
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
    pub enum ItemOrLoot {
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
        pub container_type: ContainerType,
        pub content: Vec<ItemOrLoot>,
        #[serde(default)]
        pub direction: Direction,
    }

    impl ContainerData {
        pub(super) fn place(
            &self,
            pos: Pos,
            symbol: Symbol,
            builder: &mut Builder,
        ) -> Result<(), String> {
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

pub type SymbolMap = IndexMap<char, SymbolData>;
pub type DoorPairMap = IndexMap<String, door::DoorPairData>;

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    pub areas: Vec<AreaData>,
    pub door_pairs: DoorPairMap,
}

impl LocationData {
    pub fn load_from_json(name: &str) -> Result<Self, String> {
        asset::load_json_simple(format!("location/{name}.json"))
    }

    pub fn build(self, gen_context: &mut LocationGenContext) -> Result<Pos, String> {
        let mut builder = Builder::new(gen_context, self.door_pairs);
        let base_symbols = load_base_symbols()?;

        for area in self.areas {
            area.build(&mut builder, &base_symbols)?;
        }

        builder.door_pair_builder.verify_all_doors_placed()?;

        creature::align_aggressiveness(&mut builder.gen_context.world);

        builder.get_random_entry_pos()
    }
}

#[derive(Serialize, Deserialize)]
pub struct AreaData {
    pub name: String,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub background: BackgroundId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_offset: Option<Coord>,
    pub objects: Vec<String>,
    pub symbols: SymbolMap,
}

impl AreaData {
    fn build(self, builder: &mut Builder, parent_symbols: &SymbolMap) -> Result<(), String> {
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

pub struct Symbols<'a> {
    parent_map: &'a SymbolMap,
    map: &'a SymbolMap,
}

impl<'a> Symbols<'a> {
    pub fn new(parent_map: &'a SymbolMap, map: &'a SymbolMap) -> Self {
        Self { parent_map, map }
    }

    pub fn lookup(&self, symbol: char) -> Option<&'a SymbolData> {
        self.map
            .get(&symbol)
            .or_else(|| self.parent_map.get(&symbol))
    }
}

pub fn load_base_symbols() -> Result<SymbolMap, String> {
    let file = File::open("assets/symbols.json")
        .map_err(|error| format!("Failed to open symbols file: {error}"))?;
    serde_json::from_reader::<_, SymbolMap>(file)
        .map_err(|error| format!("Failed to parse symbols file: {error}"))
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SymbolData {
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
        #[serde(default, skip_serializing_if = "crate::is_default")]
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
    fn new(gen_context: &'a mut LocationGenContext, door_pair_data: DoorPairMap) -> Self {
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
        ModelId::fortuna_chest(),
        OrderWeight::Background,
        Noun::new("fortuna chest", "fortuna chests"),
        pos,
        FortunaChest,
    ));
}
