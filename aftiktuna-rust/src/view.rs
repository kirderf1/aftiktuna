use self::area::RenderData;
pub use self::status::FullStatus;
pub(crate) use self::status::get_full_status;
use self::text::{IntoMessage, Messages};
use crate::StopType;
use crate::asset::GameAssets;
use crate::core::area::{Area, BackgroundId};
use crate::core::display::{DialogueExpression, SpeciesColorId};
use crate::core::position::{Direction, Pos};
use crate::core::status::Health;
use crate::core::store::IsTrading;
use crate::game_loop::GameState;
use crate::location::Choice;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::mem::take;

pub mod area;
mod status;
pub mod text;

mod store {
    use super::{Buffer, Frame, StatusCache, status};
    use crate::asset::NounDataMap;
    use crate::core::area::{Area, BackgroundId};
    use crate::core::display::SpeciesColorId;
    use crate::core::inventory::Held;
    use crate::core::item::{self, ItemTypeId};
    use crate::core::name::{NameData, NameIdData, NameQuery, NounData};
    use crate::core::position::Pos;
    use crate::core::store::{Shopkeeper, StockQuantity, StoreStock};
    use crate::deref_clone;
    use hecs::{Entity, World};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize)]
    pub struct StoreView {
        pub items: Vec<StoreStockView>,
        pub shopkeeper_color: Option<SpeciesColorId>,
        pub background: BackgroundId,
        pub points: i32,
        pub sellable_items: Vec<(NameData, u16)>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct StoreStockView {
        pub item_noun: NounData,
        pub price: item::Price,
        pub quantity: StockQuantity,
    }

    impl StoreStockView {
        fn create(store_stock: &StoreStock, noun_map: &NounDataMap) -> Self {
            Self {
                item_noun: noun_map.lookup(&store_stock.item.noun_id()).clone(),
                price: store_stock.price,
                quantity: store_stock.quantity,
            }
        }
    }

    pub fn store_frame(
        shopkeeper: Entity,
        buffer: &mut Buffer,
        world: &World,
        character: Entity,
        cache: &mut StatusCache,
    ) -> Frame {
        let area = world.get::<&Pos>(shopkeeper).unwrap().get_area();
        // Use the cache in shop view before status messages so that points aren't shown in status messages too
        let points = status::fetch_points(world, character, cache);
        let items = world
            .get::<&Shopkeeper>(shopkeeper)
            .unwrap()
            .0
            .iter()
            .map(|stock| StoreStockView::create(stock, &buffer.assets.noun_data_map))
            .collect();
        let mut sellable_items_count = IndexMap::new();
        world
            .query::<(&ItemTypeId, &Held, NameQuery)>()
            .iter()
            .filter(|(_, (item_type, held, _))| {
                buffer
                    .assets
                    .item_type_map
                    .get(item_type)
                    .is_some_and(|data| data.price.is_some())
                    && held.held_by(character)
            })
            .map(|(_, (_, _, query))| NameIdData::from(query))
            .for_each(|name_data| *sellable_items_count.entry(name_data).or_default() += 1);
        let sellable_items = sellable_items_count
            .into_iter()
            .map(|(name_data, count)| (name_data.lookup(buffer.assets), count))
            .collect();

        Frame::StoreView {
            view: StoreView {
                items,
                shopkeeper_color: world
                    .get::<&SpeciesColorId>(shopkeeper)
                    .ok()
                    .map(deref_clone),
                background: world.get::<&Area>(area).unwrap().background.clone(),
                points,
                sellable_items,
            },
            messages: buffer
                .pop_messages(world, character, cache)
                .into_text(buffer.assets),
        }
    }
}

pub use store::{StoreStockView, StoreView};

pub type StatusCache = status::Cache;

pub(crate) struct Buffer<'a> {
    pub messages: Messages,
    captured_frames: Vec<Frame>,
    unseen_view: bool,
    pub assets: &'a GameAssets,
}

impl<'a> Buffer<'a> {
    pub fn new(assets: &'a GameAssets) -> Self {
        Self {
            messages: Messages::default(),
            captured_frames: Vec::new(),
            unseen_view: false,
            assets,
        }
    }

    pub fn capture_view(&mut self, state: &mut GameState, ensure_command_readiness_frame: bool) {
        let frame = if let Some(shopkeeper) = state
            .world
            .get::<&IsTrading>(state.controlled)
            .ok()
            .map(|is_trading| is_trading.0)
        {
            store::store_frame(
                shopkeeper,
                self,
                &state.world,
                state.controlled,
                &mut state.status_cache,
            )
        } else {
            area_view_frame(self, state)
        };

        if self.unseen_view
            || self.captured_frames.is_empty()
            || (ensure_command_readiness_frame
                && matches!(self.captured_frames.last(), Some(Frame::Dialogue { .. })))
            || frame.has_messages()
        {
            self.captured_frames.push(frame);
        }
        self.unseen_view = false;
    }

    pub fn capture_view_before_dialogue(&mut self, state: &mut GameState) {
        if !self.messages.is_empty() {
            self.capture_view(state, false);
        }
    }

    pub fn mark_unseen_view(&mut self) {
        self.unseen_view = true;
    }

    pub fn capture_unseen_view(&mut self, state: &mut GameState) {
        if self.unseen_view {
            self.capture_view(state, false);
        }
    }

    pub fn add_change_message(&mut self, message: impl IntoMessage, state: &mut GameState) {
        self.messages.add(message);
        self.flush_hint(state);
    }

    pub fn flush_hint(&mut self, state: &mut GameState) {
        if self.messages.len() >= 4 {
            self.capture_view(state, false);
        }
    }

    fn pop_messages(
        &mut self,
        world: &World,
        character: Entity,
        cache: &mut StatusCache,
    ) -> Messages {
        let mut messages = Messages::default();
        self.pop_message_cache(&mut messages);
        status::changes_messages(world, character, &mut messages, cache, self.assets);
        messages
    }

    fn pop_message_cache(&mut self, messages: &mut Messages) {
        let messages_text = take(&mut self.messages).into_text(self.assets).join(" ");
        if !messages_text.is_empty() {
            messages.add(messages_text);
        }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.captured_frames.push(frame);
    }

    pub fn push_ending_frame(&mut self, _world: &World, _character: Entity, stop_type: StopType) {
        self.push_frame(Frame::Ending { stop_type })
    }

    pub fn push_dialogue(
        &mut self,
        world: &World,
        target: Entity,
        expression: DialogueExpression,
        message: impl AsRef<str>,
    ) {
        self.push_frame(Frame::Dialogue {
            messages: vec![format!("\"{}\"", message.as_ref())],
            data: DialogueFrameData::build(target, expression, world),
        })
    }

    pub fn into_frames(self) -> Vec<Frame> {
        self.captured_frames
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Frame {
    Introduction,
    AreaView {
        messages: Vec<String>,
        render_data: RenderData,
    },
    Dialogue {
        messages: Vec<String>,
        data: DialogueFrameData,
    },
    StoreView {
        view: StoreView,
        messages: Vec<String>,
    },
    LocationChoice(Choice),
    Error(String),
    Ending {
        stop_type: StopType,
    },
}

impl Frame {
    fn has_messages(&self) -> bool {
        match self {
            Frame::AreaView { messages, .. }
            | Frame::Dialogue { messages, .. }
            | Frame::StoreView { messages, .. } => !messages.is_empty(),
            Frame::Introduction
            | Frame::LocationChoice(_)
            | Frame::Error(_)
            | Frame::Ending { .. } => true,
        }
    }

    pub fn get_messages(&self) -> Vec<String> {
        match self {
            Frame::Introduction => intro_messages(),
            Frame::AreaView { messages, .. } => messages.clone(),
            Frame::Dialogue { messages, .. } => messages.clone(),
            Frame::StoreView { messages, .. } => messages.clone(),
            Frame::LocationChoice(choice) => choice.presentation_text_lines(),
            Frame::Error(message) => vec![message.to_owned()],
            Frame::Ending { stop_type, .. } => vec![stop_type_message(*stop_type)],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueFrameData {
    pub background: BackgroundId,
    pub color: Option<SpeciesColorId>,
    pub direction: Direction,
    pub is_badly_hurt: bool,
    pub darkness: f32,
    pub expression: DialogueExpression,
}

impl DialogueFrameData {
    fn build(character: Entity, expression: DialogueExpression, world: &World) -> Self {
        let character_ref = world.entity(character).unwrap();
        let area = character_ref.get::<&Pos>().unwrap().get_area();
        let area = world.get::<&Area>(area).unwrap();
        Self {
            background: area.background.clone(),
            color: character_ref.get::<&SpeciesColorId>().as_deref().cloned(),
            direction: character_ref
                .get::<&Direction>()
                .as_deref()
                .copied()
                .unwrap_or_default(),
            is_badly_hurt: character_ref
                .get::<&Health>()
                .is_some_and(|health| health.is_badly_hurt()),
            darkness: area.darkness,
            expression,
        }
    }
}

fn stop_type_message(stop_type: StopType) -> String {
    match stop_type {
        StopType::Win => "Congratulations, you won!",
        StopType::Lose => "You lost.",
    }
    .into()
}

fn intro_messages() -> Vec<String> {
    vec![
        "Welcome to Aftiktuna!".into(),
        "Your goal is to lead a group of aftiks on their journey through space, following the distress signal of the rumored Fortuna ship.".into(),
        "The crashed ship is said to have carried the fabled Fortuna chest, which is said to contain the item that the finder desires the most.".into(),
    ]
}

fn area_view_frame(buffer: &mut Buffer, state: &mut GameState) -> Frame {
    Frame::AreaView {
        messages: buffer
            .pop_messages(&state.world, state.controlled, &mut state.status_cache)
            .into_text(buffer.assets),
        render_data: area::prepare_render_data(state, buffer.assets),
    }
}
