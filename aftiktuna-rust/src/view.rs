use self::area::RenderData;
pub use self::status::print_full_status;
use crate::core::area::{Area, BackgroundId};
use crate::core::display::AftikColorId;
use crate::core::name::NameData;
use crate::core::position::{Direction, Pos};
use crate::core::status::Health;
use crate::core::store::IsTrading;
use crate::game_loop::{GameState, StopType};
use crate::location::Choice;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::mem::take;

pub mod area;
mod status;

mod text {
    #[derive(Default)]
    pub struct Messages(Vec<String>);

    impl Messages {
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn add(&mut self, message: impl AsRef<str>) {
            self.0.push(capitalize(message));
        }

        pub fn into_text(self) -> Vec<String> {
            self.0
        }
    }

    impl<T: AsRef<str>> From<T> for Messages {
        fn from(value: T) -> Self {
            let mut messages = Self::default();
            messages.add(value);
            messages
        }
    }

    pub fn capitalize(text: impl AsRef<str>) -> String {
        let mut chars = text.as_ref().chars();
        match chars.next() {
            None => String::new(),
            Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

pub use text::{capitalize, Messages};

mod store {
    use super::{status, text, Buffer, Frame, StatusCache};
    use crate::core::area::{Area, BackgroundId};
    use crate::core::display::AftikColorId;
    use crate::core::inventory::Held;
    use crate::core::item::Price;
    use crate::core::name::{NameData, NameQuery};
    use crate::core::position::Pos;
    use crate::core::store::{Shopkeeper, StoreStock};
    use crate::deref_clone;
    use hecs::{Entity, World};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct StoreView {
        pub items: Vec<StoreStock>,
        pub shopkeeper_color: Option<AftikColorId>,
        pub background: BackgroundId,
        pub points: i32,
        pub sellable_items: Vec<NameData>,
    }

    impl StoreView {
        pub fn push_store_view_lines(&self, text_lines: &mut Vec<String>) {
            let items = self
                .items
                .iter()
                .map(|stock| {
                    (
                        text::capitalize(stock.item.noun_data().singular()),
                        format!("{price}p", price = stock.price.buy_price()),
                        stock.quantity,
                    )
                })
                .collect::<Vec<_>>();

            let names_length = items
                .iter()
                .map(|(name, _, _)| name.len())
                .max()
                .unwrap_or(0);
            let prices_length = items
                .iter()
                .map(|(_, price, _)| price.len())
                .max()
                .unwrap_or(0);

            for (name, price, quantity) in items {
                text_lines.push(format!(
                    "{name:names_length$} | {price:prices_length$} | {quantity}"
                ));
            }
            text_lines.push(format!("Crew points: {}p", self.points));
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
        let store_info = world.get::<&Shopkeeper>(shopkeeper).unwrap();
        let items = store_info.0.clone();
        let sellable_items = world
            .query::<(&Held, NameQuery)>()
            .with::<&Price>()
            .iter()
            .filter(|(_, (held, _))| held.held_by(character))
            .map(|(_, (_, query))| NameData::from(query))
            .collect();
        Frame::StoreView {
            view: StoreView {
                items,
                shopkeeper_color: world.get::<&AftikColorId>(shopkeeper).ok().map(deref_clone),
                background: world.get::<&Area>(area).unwrap().background.clone(),
                points,
                sellable_items,
            },
            messages: buffer.pop_messages(world, character, cache).into_text(),
        }
    }
}

pub use store::StoreView;

pub type StatusCache = status::Cache;

#[derive(Default)]
pub struct Buffer {
    pub messages: Messages,
    captured_frames: Vec<Frame>,
}

impl Buffer {
    pub fn capture_view(&mut self, state: &mut GameState) {
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

        if self.captured_frames.is_empty() || frame.has_messages() {
            self.captured_frames.push(frame);
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
        status::changes_messages(world, character, &mut messages, cache);
        messages
    }

    fn pop_message_cache(&mut self, messages: &mut Messages) {
        let messages_text = take(&mut self.messages).into_text().join(" ");
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

    pub fn into_frames(self) -> Vec<Frame> {
        self.captured_frames
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    Ending {
        stop_type: StopType,
    },
}

impl Frame {
    pub fn as_text(&self) -> Vec<String> {
        let mut text_lines = Vec::new();
        match self {
            Frame::Introduction => {
                return intro_messages();
            }
            Frame::AreaView {
                messages,
                render_data,
            } => {
                text_lines.push("--------------------".into());
                area::push_area_view_lines(&mut text_lines, render_data);

                if !messages.is_empty() {
                    text_lines.push(String::default());

                    text_lines.extend(messages.clone());
                }
            }
            Frame::Dialogue { messages, data } => {
                text_lines.push(format!("{}:", capitalize(data.speaker.definite())));
                text_lines.extend(messages.clone());
            }
            Frame::StoreView { view, messages, .. } => {
                text_lines.push("--------------------".into());
                view.push_store_view_lines(&mut text_lines);

                if !messages.is_empty() {
                    text_lines.push(String::default());

                    text_lines.extend(messages.clone());
                }
            }
            Frame::LocationChoice(choice) => {
                text_lines.push("--------------------".into());
                text_lines.extend(choice.presentation_text_lines());
            }
            Frame::Ending { stop_type, .. } => {
                text_lines.push(String::default());
                text_lines.extend(stop_type_messages(*stop_type).into_text());
            }
        }
        text_lines
    }

    fn has_messages(&self) -> bool {
        match self {
            Frame::AreaView { messages, .. }
            | Frame::Dialogue { messages, .. }
            | Frame::StoreView { messages, .. } => !messages.is_empty(),
            Frame::Introduction | Frame::LocationChoice(_) | Frame::Ending { .. } => true,
        }
    }

    pub fn get_messages(&self) -> Vec<String> {
        match self {
            Frame::Introduction => intro_messages(),
            Frame::AreaView { messages, .. } => messages.clone(),
            Frame::Dialogue { messages, .. } => messages.clone(),
            Frame::StoreView { messages, .. } => messages.clone(),
            Frame::LocationChoice(choice) => choice.presentation_text_lines(),
            Frame::Ending { stop_type, .. } => stop_type_messages(*stop_type).into_text(),
        }
    }

    pub fn new_dialogue(world: &World, character: Entity, messages: Vec<String>) -> Self {
        Self::Dialogue {
            messages,
            data: DialogueFrameData::build(character, world),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueFrameData {
    pub background: BackgroundId,
    pub speaker: NameData,
    pub color: Option<AftikColorId>,
    pub direction: Direction,
    pub is_badly_hurt: bool,
}

impl DialogueFrameData {
    fn build(character: Entity, world: &World) -> Self {
        let character_ref = world.entity(character).unwrap();
        let area = character_ref.get::<&Pos>().unwrap().get_area();
        Self {
            background: world.get::<&Area>(area).unwrap().background.clone(),
            speaker: NameData::find_by_ref(character_ref),
            color: character_ref.get::<&AftikColorId>().as_deref().cloned(),
            direction: character_ref
                .get::<&Direction>()
                .as_deref()
                .copied()
                .unwrap_or_default(),
            is_badly_hurt: character_ref
                .get::<&Health>()
                .map_or(false, |health| health.is_badly_hurt()),
        }
    }
}

pub fn stop_type_messages(stop_type: StopType) -> Messages {
    match stop_type {
        StopType::Win => Messages::from("Congratulations, you won!"),
        StopType::Lose => Messages::from("You lost."),
    }
}

fn intro_messages() -> Vec<String> {
    vec!["Welcome to Aftiktuna!".to_string(),"Your goal is to lead a group of aftiks on their journey through space to find the fabled Fortuna chest, which is said to contain the item that the finder desires the most.".to_string()]
}

fn area_view_frame(buffer: &mut Buffer, state: &mut GameState) -> Frame {
    Frame::AreaView {
        messages: buffer
            .pop_messages(&state.world, state.controlled, &mut state.status_cache)
            .into_text(),
        render_data: area::prepare_render_data(state),
    }
}
