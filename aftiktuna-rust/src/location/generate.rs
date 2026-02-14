pub(super) mod creature;
pub(super) mod door;

use super::LocationGenContext;
use crate::asset::location::{
    AreaData, ContainerData, DoorPairMap, FurnishTemplate, ItemOrLoot, LocationData, SymbolData,
    SymbolLookup, SymbolMap,
};
use crate::asset::{self, loot};
use crate::core::FortunaChest;
use crate::core::area::{Area, ShipControls};
use crate::core::display::ModelId;
use crate::core::inventory::{Container, Held};
use crate::core::name::NounId;
use crate::core::position::{Coord, Pos};
use hecs::{Entity, World};
use rand::seq::IndexedRandom;

pub struct LocationBuildData {
    pub spawned_areas: Vec<Entity>,
    pub entry_pos: Pos,
    pub food_deposit_pos: Option<Pos>,
    pub ship_dialogue_spot: Option<Pos>,
}

pub fn build_location<'a, 'b>(
    location_data: LocationData,
    gen_context: &'a mut LocationGenContext<'b>,
) -> Result<LocationBuildData, String> {
    let variant = location_data
        .variants
        .choose_weighted(&mut gen_context.rng, |variant| variant.weight)
        .ok()
        .map(|variant| variant.id.clone());
    let mut builder = Builder::new(gen_context, variant, location_data.door_pairs);
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
        ship_dialogue_spot: builder.ship_dialogue_spot,
    })
}

fn build_area(
    area_data: AreaData,
    builder: &mut Builder,
    base_symbols: &SymbolMap,
) -> Result<Entity, String> {
    let area = builder.gen_context.world.spawn((Area {
        size: area_data.objects.len().try_into().unwrap(),
        label: area_data.name,
        background: area_data.background,
        background_offset: area_data.background_offset.unwrap_or(0),
        extra_background_layers: area_data.extra_background_layers,
        darkness: area_data.darkness,
    },));
    if let Some(tag) = area_data.tag {
        builder.gen_context.world.insert_one(area, tag).unwrap();
    }

    let symbols = SymbolLookup::new(base_symbols, &area_data.symbols);

    for (coord, objects) in area_data.objects.iter().enumerate() {
        let pos = Pos::new(area, coord as Coord, &builder.gen_context.world);
        for symbol in objects.chars() {
            match symbols.lookup(symbol) {
                Some(symbol_data) => place_symbol(symbol_data, pos, builder, base_symbols)?,
                None => Err(format!("Unknown symbol \"{symbol}\""))?,
            }
        }
    }

    if let Some(variant) = builder.chosen_variant.as_ref()
        && let Some(objects) = area_data.variant_objects.get(variant)
    {
        if objects.len() != area_data.objects.len() {
            return Err(format!(
                "Variant \"{variant}\" with objects size {} does not match area size {}",
                objects.len(),
                area_data.objects.len()
            ));
        }
        for (coord, objects) in objects.iter().enumerate() {
            let pos = Pos::new(area, coord as Coord, &builder.gen_context.world);
            for symbol in objects.chars() {
                match symbols.lookup(symbol) {
                    Some(symbol_data) => place_symbol(symbol_data, pos, builder, base_symbols)?,
                    None => Err(format!("Unknown symbol \"{symbol}\""))?,
                }
            }
        }
    }

    Ok(area)
}

fn place_symbol(
    symbol_data: &SymbolData,
    pos: Pos,
    builder: &mut Builder,
    base_symbols: &SymbolMap,
) -> Result<(), String> {
    match symbol_data {
        SymbolData::LocationEntry => builder.add_entry_pos(pos),
        SymbolData::FortunaChest => place_fortuna_chest(&mut builder.gen_context.world, pos),
        SymbolData::ShipControls { direction } => {
            builder.gen_context.world.spawn((
                ModelId::ship_controls(),
                NounId::from("ship_controls"),
                pos,
                *direction,
                ShipControls,
            ));
        }
        SymbolData::FoodDeposit => {
            if builder.food_deposit_pos.is_some() {
                return Err("Can only place one food deposit per location".to_owned());
            } else {
                builder.food_deposit_pos = Some(pos);
            }
        }
        SymbolData::ShipDialogueSpot => {
            if builder.ship_dialogue_spot.is_some() {
                return Err("Can only place one ship dialogue spot per location".to_owned());
            } else {
                builder.ship_dialogue_spot = Some(pos);
            }
        }
        SymbolData::Item { item } => {
            item.spawn(&mut builder.gen_context.world, pos);
        }
        SymbolData::Loot { table } => {
            let item_type = builder
                .loot_table_cache
                .get_or_load(table)?
                .pick_loot_item(&mut builder.gen_context.rng);
            item_type.spawn(&mut builder.gen_context.world, pos);
        }
        SymbolData::Door(door_data) => door::place(door_data, pos, builder)?,
        SymbolData::Inanimate { model, direction } => {
            builder
                .gen_context
                .world
                .spawn((model.clone(), pos, *direction));
        }
        SymbolData::Container(container_data) => place_container(container_data, pos, builder)?,
        SymbolData::Creature(creature_data) => {
            creature::place_creature(creature_data, pos, builder.gen_context)?
        }
        SymbolData::Character(npc_data) => creature::place_npc(
            npc_data,
            pos,
            builder.gen_context,
            &mut builder.loot_table_cache,
        )?,
        SymbolData::CharacterCorpse(aftik_corpse_data) => {
            creature::place_corpse(aftik_corpse_data, pos, builder.gen_context)?
        }
        SymbolData::Furnish { template } => {
            let template_list = FurnishTemplate::load_list(template)?;
            let template_data = template_list
                .choose(&mut builder.gen_context.rng)
                .ok_or_else(|| format!("Furnish template \"{template}\" is without entries."))?;
            furnish(template_data, pos, builder, base_symbols)
                .map_err(|error| format!("Error from furnish template \"{template}\": {error}"))?;
        }
    }
    Ok(())
}

struct Builder<'a, 'b> {
    gen_context: &'a mut LocationGenContext<'b>,
    chosen_variant: Option<String>,
    entry_positions: Vec<Pos>,
    food_deposit_pos: Option<Pos>,
    ship_dialogue_spot: Option<Pos>,
    door_pair_builder: door::DoorPairsBuilder,
    loot_table_cache: loot::LootTableCache,
}

impl<'a, 'b> Builder<'a, 'b> {
    fn new(
        gen_context: &'a mut LocationGenContext<'b>,
        chosen_variant: Option<String>,
        door_pair_data: DoorPairMap,
    ) -> Self {
        Self {
            gen_context,
            chosen_variant,
            entry_positions: Vec::new(),
            food_deposit_pos: None,
            ship_dialogue_spot: None,
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

fn place_fortuna_chest(world: &mut World, pos: Pos) {
    world.spawn((
        ModelId::fortuna_chest(),
        NounId::from("fortuna_chest"),
        pos,
        FortunaChest,
    ));
}

fn place_container(data: &ContainerData, pos: Pos, builder: &mut Builder) -> Result<(), String> {
    let container = builder.gen_context.world.spawn((
        data.container_type.model_id(),
        data.container_type.noun_id(),
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
    let item_type = match item_or_loot {
        ItemOrLoot::Item { item } => item,
        ItemOrLoot::Loot { table } => builder
            .loot_table_cache
            .get_or_load(table)?
            .pick_loot_item(&mut builder.gen_context.rng),
    };
    item_type.spawn(
        &mut builder.gen_context.world,
        Held::in_inventory(container),
    );
    Ok(())
}

fn furnish(
    furnish_template: &FurnishTemplate,
    pos: Pos,
    builder: &mut Builder,
    base_symbols: &SymbolMap,
) -> Result<(), String> {
    let symbols = SymbolLookup::new(base_symbols, &furnish_template.symbols);

    for (coord, objects) in furnish_template.objects.iter().enumerate() {
        let pos = pos
            .try_offset(coord as i32, &builder.gen_context.world)
            .ok_or_else(|| "Too large template".to_string())?;
        for symbol in objects.chars() {
            match symbols.lookup(symbol) {
                Some(symbol_data) => place_symbol(symbol_data, pos, builder, base_symbols)?,
                None => Err(format!("Unknown symbol \"{symbol}\""))?,
            }
        }
    }
    Ok(())
}
