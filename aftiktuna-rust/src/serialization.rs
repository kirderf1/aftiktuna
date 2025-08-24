use crate::game_interface::{Game, SerializedState};
use hecs::World;
use rmp_serde::{decode, encode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{Read, Write};

pub const SAVE_FILE_NAME: &str = "SAVE_FILE";
const MAJOR_VERSION: u16 = 5;
const MINOR_VERSION: u16 = 0;

fn verify_version(major: u16, minor: u16) -> Result<(), LoadError> {
    if major != MAJOR_VERSION || minor > MINOR_VERSION {
        Err(LoadError::UnsupportedVersion(major, minor))
    } else {
        Ok(())
    }
}

macro_rules! world_serialization {
    ($($comp:ty, $id:ident);* $(;)?) => {
        use hecs::serialize::column;
        use hecs::{Archetype, ColumnBatchBuilder, ColumnBatchType, World};
        use serde::de::SeqAccess;
        use serde::ser::SerializeTuple;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};
        use std::any::TypeId;

        pub fn serialize<S: Serializer>(world: &World, serializer: S) -> Result<S::Ok, S::Error>
        {
            column::serialize(world, &mut HecsSerializeContext, serializer)
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<World, D::Error>
        {
            column::deserialize(&mut HecsDeserializeContext::default(), deserializer)
        }

        #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
        pub(super) enum ComponentId {
            $(
            $id
            ),*
        }
        pub(super) fn is_serialized_type_id(id: TypeId) -> bool {
            $(id == TypeId::of::<$comp>())||*
        }
        pub(super) fn all_component_ids() -> Vec<ComponentId> {
            vec![
                $(ComponentId::$id,)*
            ]
        }
        impl From<ComponentId> for TypeId {
            fn from(value: ComponentId) -> Self {
                match value {
                    $(ComponentId::$id => TypeId::of::<$comp>(),)*
                }
            }
        }

        struct HecsSerializeContext;

        impl column::SerializeContext for HecsSerializeContext {
            fn component_count(&self, archetype: &Archetype) -> usize {
                archetype
                    .component_types()
                    .filter(|&id| is_serialized_type_id(id))
                    .count()
            }

            fn serialize_component_ids<S: SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                column::try_serialize_id::<$comp, _, _>(archetype, &ComponentId::$id, &mut out)?;
                )*
                out.end()
            }

            fn serialize_components<S: SerializeTuple>(
                &mut self,
                archetype: &Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                column::try_serialize::<$comp, _>(archetype, &mut out)?;
                )*
                out.end()
            }
        }

        #[derive(Default)]
        struct HecsDeserializeContext(Vec<ComponentId>);

        impl column::DeserializeContext for HecsDeserializeContext {
            fn deserialize_component_ids<'de, A>(
                &mut self,
                mut seq: A,
            ) -> Result<ColumnBatchType, A::Error>
            where
                A: SeqAccess<'de>,
            {
                self.0.clear();
                let mut batch = ColumnBatchType::new();
                while let Some(id) = seq.next_element()? {
                    match id {
                        $(
                        ComponentId::$id => {
                            batch.add::<$comp>();
                        }
                        )*
                    }
                    self.0.push(id);
                }
                Ok(batch)
            }

            fn deserialize_components<'de, A>(
                &mut self,
                entity_count: u32,
                mut seq: A,
                batch: &mut ColumnBatchBuilder,
            ) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                for &id in &self.0 {
                    match id {
                        $(
                        ComponentId::$id => {
                            column::deserialize_column::<$comp, _>(entity_count, &mut seq, batch)?;
                        }
                        )*
                    }
                }
                Ok(())
            }
        }
    };
}

pub mod world {
    use crate::core::{
        self, area, behavior, display, inventory, item, name, position, status, store,
    };
    use std::error::Error;

    world_serialization!(
        area::Area, Area;
        area::ShipState, ShipState;
        area::ShipRoom, ShipRoom;
        area::ShipControls, ShipControls;
        position::Pos, Pos;
        position::Direction, Direction;
        position::Large, Large;
        position::OccupiesSpace, OccupiesSpace;

        core::Species, Species;

        name::Name, Name;
        name::NounId, NounId;
        name::Adjective, Adjective;
        display::ModelId, TextureType;
        display::OrderWeight, OrderWeight;
        display::AftikColorId, AftikColor;

        status::Stats, Stats;
        status::CreatureAttribute, CreatureAttribute;
        status::Traits, Traits;
        status::Health, Health;
        status::Stamina, Stamina;
        status::SeenWithLowHealth, LowHealth;
        status::SeenWithLowStamina, LowStamina;
        status::IsStunned, IsStunned;

        core::CrewMember, CrewMember;
        behavior::Character, Character;
        behavior::Hostile, Hostile;
        behavior::Wandering, Wandering;
        behavior::ObservationTarget, ObservationTarget;
        behavior::Intention, Intention;
        behavior::RepeatingAction, RepeatingAction;
        behavior::Waiting, Waiting;
        behavior::CrewLossMemory, CrewLossMemory;

        behavior::Recruitable, Recruitable;
        behavior::GivesHuntReward, GivesHuntReward;
        behavior::EncounterDialogue, EncounterDialogue;
        core::Tag, Tag;

        store::Shopkeeper, Shopkeeper;
        store::IsTrading, IsTrading;
        store::Points, Points;

        core::Door, Door;
        core::IsCut, IsCut;
        core::BlockType, BlockType;

        inventory::Held, Held;
        inventory::Container, Container;

        item::ItemType, ItemType;
        item::CanWield, CanWield;
        item::Price, Price;

        core::FortunaChest, FortunaChest;
        core::OpenedChest, OpenedChest;
    );

    struct WorldSerialize<'a>(&'a World);

    impl<'a> Serialize for WorldSerialize<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serialize(self.0, serializer)
        }
    }

    struct WorldDeserialize(World);

    impl<'de> Deserialize<'de> for WorldDeserialize {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize(deserializer).map(WorldDeserialize)
        }
    }

    pub fn serialize_clone(world: &World) -> Result<World, Box<dyn Error>> {
        let world_data = rmp_serde::to_vec(&WorldSerialize(world))?;
        let WorldDeserialize(world_clone) = rmp_serde::from_slice(&world_data)?;
        Ok(world_clone)
    }
}

macro_rules! from {
    ($other_error:ty => $error:ty, $variant:expr) => {
        impl From<$other_error> for $error {
            fn from(value: $other_error) -> Self {
                $variant(value)
            }
        }
    };
}

pub enum SaveError {
    IO(io::Error),
    Encode(encode::Error),
}

from!(io::Error => SaveError, SaveError::IO);
from!(encode::Error => SaveError, SaveError::Encode);

impl Display for SaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::IO(error) => Display::fmt(error, f),
            SaveError::Encode(error) => Display::fmt(error, f),
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    UnsupportedVersion(u16, u16),
    Decode(decode::Error),
    Io(io::Error),
}

from!(decode::Error => LoadError, LoadError::Decode);
from!(io::Error => LoadError, LoadError::Io);

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::UnsupportedVersion(major, minor) => write!(
                f,
                "Unsupported save file format '{major}.{minor}'. Current format version is '{MAJOR_VERSION}.{MINOR_VERSION}'."
            ),
            LoadError::Decode(error) => Display::fmt(error, f),
            LoadError::Io(error) => Display::fmt(error, f),
        }
    }
}

pub fn write_game_to_save_file(game: &Game) -> Result<(), SaveError> {
    serialize_game(&game.serialized_state, File::create(SAVE_FILE_NAME)?)
}

fn serialize_game(state: &SerializedState, writer: impl Write) -> Result<(), SaveError> {
    let mut serializer = rmp_serde::Serializer::new(writer).with_struct_map();
    (MAJOR_VERSION, MINOR_VERSION).serialize(&mut serializer)?;
    state.serialize(&mut serializer)?;
    Ok(())
}

pub(crate) fn load_game(reader: impl Read) -> Result<SerializedState, LoadError> {
    let mut deserializer = rmp_serde::Deserializer::new(reader);
    let (major, minor) = <(u16, u16)>::deserialize(&mut deserializer)?;
    verify_version(major, minor)?;
    Ok(SerializedState::deserialize(&mut deserializer)?)
}

pub fn check_world_components(world: &World) {
    let mut set = HashSet::new();
    for archetype in world.archetypes() {
        if archetype.is_empty() {
            continue;
        }
        for component_type in archetype.component_types() {
            set.insert(component_type);
        }
    }

    let absent_serialized_components = world::all_component_ids()
        .into_iter()
        .filter(|&component_id| !set.contains(&component_id.into()))
        .collect::<Vec<_>>();
    if !absent_serialized_components.is_empty() {
        println!("Has unused serialized components: {absent_serialized_components:?}");
    }
    let non_serialized_components = set
        .into_iter()
        .filter(|component_type| !world::is_serialized_type_id(*component_type))
        .collect::<Vec<_>>();
    if !non_serialized_components.is_empty() {
        println!("Has non-serialized components: {non_serialized_components:?}");
    }
}
