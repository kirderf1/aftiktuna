pub(super) mod creature;
pub(super) mod door;

use super::LocationGenContext;
use crate::asset::location::{
    AreaData, ContainerData, DoorPairMap, ItemOrLoot, LocationData, SymbolData, SymbolLookup,
    SymbolMap,
};
use crate::asset::{self, loot};
use crate::core::FortunaChest;
use crate::core::area::{Area, ShipControls};
use crate::core::display::{ModelId, OrderWeight, Symbol};
use crate::core::inventory::{Container, Held};
use crate::core::name::Noun;
use crate::core::position::{Coord, Pos};
use hecs::{Entity, World};
use rand::seq::IndexedRandom;

pub struct LocationBuildData {
    pub spawned_areas: Vec<Entity>,
    pub entry_pos: Pos,
    pub food_deposit_pos: Option<Pos>,
}

pub fn build_location(
    location_data: LocationData,
    gen_context: &mut LocationGenContext,
) -> Result<LocationBuildData, String> {
    let mut builder = Builder::new(gen_context, location_data.door_pairs);
    let base_symbols = asset::location::load_base_symbols()?;

    let mut spawned_areas = Vec::with_capacity(location_data.areas.len());
    for area in location_data.areas {
        let area = build_area(area, &mut builder, &base_symbols)?;
        spawned_areas.push(area);
    }

    builder.door_pair_builder.verify_all_doors_placed()?;

    creature::align_aggressiveness(&mut builder.gen_context.world);

    let entry_pos = builder.get_random_entry_pos()?;

    Ok(LocationBuildData {
        spawned_areas,
        entry_pos,
        food_deposit_pos: builder.food_deposit_pos,
    })
}

fn build_area(
    area_data: AreaData,
    builder: &mut Builder,
    parent_symbols: &SymbolMap,
) -> Result<Entity, String> {
    let area = builder.gen_context.world.spawn((Area {
        size: area_data.objects.len().try_into().unwrap(),
        label: area_data.name,
        background: area_data.background,
        background_offset: area_data.background_offset.unwrap_or(0),
        extra_background_layers: area_data.extra_background_layers,
        darkness: area_data.darkness,
    },));

    let symbols = SymbolLookup::new(parent_symbols, &area_data.symbols);

    for (coord, objects) in area_data.objects.iter().enumerate() {
        let pos = Pos::new(area, coord as Coord, &builder.gen_context.world);
        for symbol in objects.chars() {
            match symbols.lookup(symbol) {
                Some(symbol_data) => place_symbol(symbol_data, pos, Symbol(symbol), builder)?,
                None => Err(format!("Unknown symbol \"{symbol}\""))?,
            }
        }
    }
    Ok(area)
}

fn place_symbol(
    symbol_data: &SymbolData,
    pos: Pos,
    symbol: Symbol,
    builder: &mut Builder,
) -> Result<(), String> {
    match symbol_data {
        SymbolData::LocationEntry => builder.add_entry_pos(pos),
        SymbolData::FortunaChest => {
            place_fortuna_chest(&mut builder.gen_context.world, symbol, pos)
        }
        SymbolData::ShipControls { direction } => {
            builder.gen_context.world.spawn((
                symbol,
                ModelId::ship_controls(),
                OrderWeight::Background,
                Noun::new("ship controls", "ship controls"),
                pos,
                *direction,
                ShipControls,
            ));
        }
        SymbolData::FoodDeposit => {
            if builder.food_deposit_pos.is_some() {
                return Err("Can only place one food deposit per location".to_string());
            } else {
                builder.food_deposit_pos = Some(pos);
            }
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
        SymbolData::Door(door_data) => door::place(door_data, pos, symbol, builder)?,
        SymbolData::Inanimate { model, direction } => {
            builder.gen_context.world.spawn((
                symbol,
                model.clone(),
                OrderWeight::Background,
                pos,
                *direction,
            ));
        }
        SymbolData::Container(container_data) => {
            place_container(container_data, pos, symbol, builder)?
        }
        SymbolData::Creature(creature_data) => {
            creature::place_creature(creature_data, pos, symbol, builder.gen_context)
        }
        SymbolData::Shopkeeper(shopkeeper_data) => {
            creature::place_shopkeeper(shopkeeper_data, pos, &mut builder.gen_context.world)?
        }
        SymbolData::Character(npc_data) => creature::place_npc(npc_data, pos, builder.gen_context),
        SymbolData::AftikCorpse(aftik_corpse_data) => {
            creature::place_corpse(aftik_corpse_data, pos, builder.gen_context)
        }
    }
    Ok(())
}

struct Builder<'a> {
    gen_context: &'a mut LocationGenContext,
    entry_positions: Vec<Pos>,
    food_deposit_pos: Option<Pos>,
    door_pair_builder: door::DoorPairsBuilder,
    loot_table_cache: loot::LootTableCache,
}

impl<'a> Builder<'a> {
    fn new(gen_context: &'a mut LocationGenContext, door_pair_data: DoorPairMap) -> Self {
        Self {
            gen_context,
            entry_positions: Vec::new(),
            food_deposit_pos: None,
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

fn place_container(
    data: &ContainerData,
    pos: Pos,
    symbol: Symbol,
    builder: &mut Builder,
) -> Result<(), String> {
    let container = builder.gen_context.world.spawn((
        symbol,
        data.container_type.model_id(),
        OrderWeight::Background,
        data.container_type.noun(),
        pos,
        data.direction,
        Container,
    ));
    for entry in &data.content {
        generate_item_or_loot(entry, container, builder)?;
    }
    Ok(())
}

fn generate_item_or_loot(
    item_or_loot: &ItemOrLoot,
    container: Entity,
    builder: &mut Builder,
) -> Result<(), String> {
    let item = match item_or_loot {
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
