pub mod creature {
    use crate::asset::profile::ProfileOrRandom;
    use crate::core::behavior::{self, BackgroundDialogue, GivesHuntRewardData, Reward};
    use crate::core::display::SpeciesColorId;
    use crate::core::item::{self, ItemTypeId};
    use crate::core::position::Direction;
    use crate::core::status::{CreatureAttribute, Stats};
    use crate::core::store::StockQuantity;
    use crate::core::{DialogueId, Species, Tag};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum AttributeChoice {
        None,
        #[default]
        Random,
        #[serde(untagged)]
        Attribute(CreatureAttribute),
    }

    impl AttributeChoice {
        pub fn variants() -> impl Iterator<Item = Self> {
            [Self::None, Self::Random].into_iter().chain(
                CreatureAttribute::variants()
                    .iter()
                    .copied()
                    .map(Self::Attribute),
            )
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Type {
        Goblin,
        Eyesaur,
        Azureclops,
        Scarvie,
        VoraciousFrog,
        BloodMantis,
    }

    impl Type {
        pub fn variants() -> &'static [Self] {
            use Type::*;
            &[
                Goblin,
                Eyesaur,
                Azureclops,
                Scarvie,
                VoraciousFrog,
                BloodMantis,
            ]
        }

        pub fn species(self) -> Species {
            match self {
                Self::Goblin => Species::Goblin,
                Self::Eyesaur => Species::Eyesaur,
                Self::Azureclops => Species::Azureclops,
                Self::Scarvie => Species::Scarvie,
                Self::VoraciousFrog => Species::VoraciousFrog,
                Self::BloodMantis => Species::BloodMantis,
            }
        }

        pub fn is_aggressive_by_default(self) -> bool {
            match self {
                Self::Goblin | Self::Eyesaur | Self::Scarvie => false,
                Self::Azureclops | Self::VoraciousFrog | Self::BloodMantis => true,
            }
        }

        pub fn is_tameable(self) -> bool {
            matches!(self, Self::Eyesaur | Self::Scarvie)
        }
    }

    pub use crate::core::behavior::Wandering;

    #[derive(Clone, Serialize, Deserialize)]
    pub struct CreatureSpawnData {
        pub creature: Type,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(default = "full_health", skip_serializing_if = "is_full_health")]
        pub health: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub stats: Option<Stats>,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        pub attribute: AttributeChoice,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub aggressive: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub wandering: Option<Wandering>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub tag: Option<Tag>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    fn full_health() -> f32 {
        1.
    }

    fn is_full_health(health: &f32) -> bool {
        *health == 1.
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct GivesHuntReward {
        pub target_tag: Tag,
        pub target_label: String,
        pub task_dialogue: DialogueId,
        pub already_completed_dialogue: DialogueId,
        pub reward_dialogue: DialogueId,
        pub reward: Reward,
    }

    impl GivesHuntReward {
        pub(crate) fn cloned_data(&self) -> GivesHuntRewardData {
            let Self {
                target_tag,
                target_label,
                task_dialogue,
                already_completed_dialogue,
                reward_dialogue,
                reward,
            } = self.clone();
            GivesHuntRewardData {
                target_tag,
                target_label,
                task_dialogue,
                already_completed_dialogue,
                reward_dialogue,
                reward,
                presented: false,
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum CharacterInteraction {
        Recruitable,
        Talk {
            dialogue: DialogueId,
        },
        GivesHuntReward(Box<GivesHuntReward>),
        Shopkeeper {
            stock: Vec<StockDefinition>,
        },
        Hostile {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            encounter_dialogue: Option<DialogueId>,
        },
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct NpcSpawnData {
        pub profile: ProfileOrRandom,
        #[serde(default = "full_health", skip_serializing_if = "is_full_health")]
        pub health: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub tag: Option<Tag>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub background: Option<behavior::BackgroundId>,
        pub interaction: CharacterInteraction,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub background_dialogue: Option<BackgroundDialogue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub wielded_item: Option<ItemTypeId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct AftikCorpseData {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub color: Option<SpeciesColorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub direction: Option<Direction>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StockDefinition {
        pub item: ItemTypeId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub price: Option<item::Price>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub quantity: Option<StockQuantity>,
    }
}

use crate::asset::background::ParallaxLayer;
use crate::asset::loot::{self, LootTableId};
use crate::core::area::BackgroundId;
use crate::core::display::ModelId;
use crate::core::item::ItemTypeId;
use crate::core::name::NounId;
use crate::core::position::Direction;
use crate::core::{BlockType, DoorKind, Tag};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::File;

pub type SymbolMap = IndexMap<char, SymbolData>;
pub type DoorPairMap = IndexMap<String, DoorPairData>;

#[derive(Serialize, Deserialize)]
pub struct WeightedVariant {
    pub id: String,
    pub weight: u32,
}

#[derive(Serialize, Deserialize)]
pub struct LocationData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<WeightedVariant>,
    pub areas: Vec<AreaData>,
    pub door_pairs: DoorPairMap,
}

impl LocationData {
    pub fn load_from_json(name: &str) -> Result<Self, String> {
        super::load_json_asset(format!("location/{name}.json")).map_err(|error| error.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AreaData {
    pub name: String,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub pos_in_overview: (i32, i32),
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<Tag>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub background: BackgroundId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_offset: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_background_layers: Vec<ParallaxLayer<String>>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub darkness: f32,
    pub objects: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub variant_objects: IndexMap<String, Vec<String>>,
    pub symbols: SymbolMap,
}

pub fn load_base_symbols() -> Result<SymbolMap, String> {
    let file = File::open("assets/symbols.json")
        .map_err(|error| format!("Failed to open symbols file: {error}"))?;
    serde_json::from_reader::<_, SymbolMap>(file)
        .map_err(|error| format!("Failed to parse symbols file: {error}"))
}

pub struct SymbolLookup<'a> {
    parent_map: &'a SymbolMap,
    map: &'a SymbolMap,
}

impl<'a> SymbolLookup<'a> {
    pub fn new(parent_map: &'a SymbolMap, map: &'a SymbolMap) -> Self {
        Self { parent_map, map }
    }

    pub fn lookup(&self, symbol: char) -> Option<&'a SymbolData> {
        self.map
            .get(&symbol)
            .or_else(|| self.parent_map.get(&symbol))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SymbolData {
    LocationEntry,
    FortunaChest,
    ShipControls {
        direction: Direction,
    },
    FoodDeposit,
    Item {
        item: ItemTypeId,
    },
    Loot {
        table: loot::LootTableId,
    },
    Door(DoorSpawnData),
    Inanimate {
        model: ModelId,
        #[serde(default, skip_serializing_if = "crate::is_default")]
        direction: Direction,
    },
    Container(ContainerData),
    Creature(creature::CreatureSpawnData),
    Character(Box<creature::NpcSpawnData>),
    AftikCorpse(creature::AftikCorpseData),
    Furnish {
        template: String,
    },
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DoorPairData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_type: Option<BlockType>,
    #[serde(default, skip_serializing_if = "crate::is_default")]
    pub is_cut: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorType {
    Door,
    Doorway,
    UpwardStairs,
    DownwardStairs,
    Shack,
    House,
    Store,
    Path,
    LeftPath,
    RightPath,
    CrossroadPath,
}

impl From<DoorType> for ModelId {
    fn from(value: DoorType) -> Self {
        ModelId::new(match value {
            DoorType::Door | DoorType::House | DoorType::Store => "door/door",
            DoorType::Doorway => "door/doorway",
            DoorType::UpwardStairs => "door/upward_stairs",
            DoorType::DownwardStairs => "door/downward_stairs",
            DoorType::Shack => "door/shack",
            DoorType::Path => "path",
            DoorType::LeftPath => "path/left_corner",
            DoorType::RightPath => "path/right_corner",
            DoorType::CrossroadPath => "path/crossroad",
        })
    }
}

impl From<DoorType> for DoorKind {
    fn from(value: DoorType) -> Self {
        match value {
            DoorType::Door
            | DoorType::Doorway
            | DoorType::UpwardStairs
            | DoorType::DownwardStairs
            | DoorType::Shack
            | DoorType::House
            | DoorType::Store => DoorKind::Door,
            DoorType::Path | DoorType::LeftPath | DoorType::RightPath | DoorType::CrossroadPath => {
                DoorKind::Path
            }
        }
    }
}

impl DoorType {
    pub fn variants() -> &'static [Self] {
        use DoorType::*;
        &[
            Door,
            Doorway,
            UpwardStairs,
            DownwardStairs,
            Shack,
            House,
            Store,
            Path,
            LeftPath,
            RightPath,
            CrossroadPath,
        ]
    }

    pub fn noun_id(self) -> NounId {
        match self {
            Self::Door => "door",
            Self::Doorway => "doorway",
            Self::UpwardStairs => "upward_stairs",
            Self::DownwardStairs => "downward_stairs",
            Self::Shack => "shack",
            Self::House => "house",
            Self::Store => "store",
            Self::Path | Self::LeftPath | Self::RightPath | Self::CrossroadPath => "path",
        }
        .into()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorAdjective {
    Left,
    Middle,
    Right,
}

impl DoorAdjective {
    pub fn variants() -> &'static [Self] {
        use DoorAdjective::*;
        &[Left, Middle, Right]
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Middle => "middle",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DoorSpawnData {
    pub pair_id: String,
    pub display_type: DoorType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjective: Option<DoorAdjective>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerType {
    Tent,
    Cabinet,
    Drawer,
    Crate,
    Chest,
    CrashedShip,
}

impl ContainerType {
    pub fn variants() -> &'static [Self] {
        use ContainerType::*;
        &[Tent, Cabinet, Drawer, Crate, Chest, CrashedShip]
    }

    pub fn model_id(self) -> ModelId {
        ModelId::new(match self {
            Self::Tent => "container/tent",
            Self::Cabinet => "container/cabinet",
            Self::Drawer => "container/drawer",
            Self::Crate => "container/crate",
            Self::Chest => "container/chest",
            Self::CrashedShip => "container/crashed_ship",
        })
    }

    pub fn noun_id(self) -> NounId {
        match self {
            Self::Tent => "tent",
            Self::Cabinet => "cabinet",
            Self::Drawer => "drawer",
            Self::Crate => "crate",
            Self::Chest => "chest",
            Self::CrashedShip => "crashed_ship",
        }
        .into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemOrLoot {
    Item { item: ItemTypeId },
    Loot { table: LootTableId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerData {
    pub container_type: ContainerType,
    pub content: Vec<ItemOrLoot>,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct FurnishTemplate {
    pub objects: Vec<String>,
    pub symbols: SymbolMap,
}

impl FurnishTemplate {
    pub fn load_list(template: &str) -> Result<Vec<Self>, String> {
        super::load_json_asset(format!("area_furnish/{template}.json"))
            .map_err(|error| error.to_string())
    }
}
