use crate::action::trade::IsTrading;
use crate::core::area::{Area, BackgroundType};
use crate::core::position::{Direction, Pos};
use crate::core::{GameState, StopType};
use crate::location::Choice;
use area::{AftikColor, RenderData};
use hecs::{Entity, World};
use name::NameData;
use serde::{Deserialize, Serialize};
pub use status::print_full_status;
use std::mem::take;

pub mod area;
pub mod name;
mod status;

mod text {
    #[derive(Default)]
    pub struct Messages(pub Vec<String>);

    impl Messages {
        pub fn add(&mut self, message: impl AsRef<str>) {
            self.0.push(capitalize(message));
        }

        pub fn print_lines(self) {
            for line in self.0 {
                println!("{line}");
            }
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
    use super::area::AftikColor;
    use super::name::{NameData, NameQuery};
    use super::{status, text, Buffer, Frame, Messages, StatusCache};
    use crate::action::trade::{PricedItem, Shopkeeper};
    use crate::core::area::{Area, BackgroundType};
    use crate::core::inventory::Held;
    use crate::core::item::Price;
    use crate::core::position::Pos;
    use hecs::{Entity, World};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct StoreView {
        pub items: Vec<PricedItem>,
        pub shopkeeper_color: Option<AftikColor>,
        pub background: BackgroundType,
        pub points: i32,
        pub sellable_items: Vec<NameData>,
    }

    impl StoreView {
        pub fn messages(&self) -> Messages {
            let mut messages = Messages::default();
            let items = self
                .items
                .iter()
                .map(|priced| {
                    (
                        text::capitalize(priced.item.noun_data().singular()),
                        priced.price,
                    )
                })
                .collect::<Vec<_>>();
            let max_length = items.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
            for (name, price) in items {
                messages.add(format!(
                    "{} {}| {}p",
                    name,
                    " ".repeat(max_length - name.len()),
                    price
                ));
            }
            messages.add(format!("Crew points: {}p", self.points));
            messages
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
                shopkeeper_color: world
                    .get::<&AftikColor>(shopkeeper)
                    .ok()
                    .map(|color| *color),
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
        let messages_text = take(&mut self.messages).0.join(" ");
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
        background: BackgroundType,
        speaker: NameData,
        color: Option<AftikColor>,
        direction: Direction,
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
        let mut text = Vec::new();
        match self {
            Frame::Introduction => {
                return intro_messages();
            }
            Frame::AreaView {
                messages,
                render_data,
            } => {
                text.push("--------------------".to_string());
                text.extend(area::area_view_messages(render_data).0);

                if !messages.is_empty() {
                    text.push(String::default());

                    text.extend(messages.clone());
                }
            }
            Frame::Dialogue {
                messages, speaker, ..
            } => {
                text.push(format!("{}:", capitalize(speaker.definite())));
                text.extend(messages.clone());
            }
            Frame::StoreView { view, messages, .. } => {
                text.push("--------------------".to_string());
                text.extend(view.messages().0);

                if !messages.is_empty() {
                    text.push(String::default());

                    text.extend(messages.clone());
                }
            }
            Frame::LocationChoice(choice) => {
                text.push("--------------------".to_string());
                text.extend(choice.present().0);
            }
            Frame::Ending { stop_type, .. } => {
                text.push(String::default());
                text.extend(stop_type.messages().into_text());
            }
        }
        text
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
            Frame::LocationChoice(choice) => choice.present().0,
            Frame::Ending { stop_type, .. } => stop_type.messages().0,
        }
    }

    pub fn new_dialogue(world: &World, character: Entity, messages: Vec<String>) -> Self {
        let area = world.get::<&Pos>(character).unwrap().get_area();
        Self::Dialogue {
            messages,
            background: world.get::<&Area>(area).unwrap().background.clone(),
            speaker: NameData::find(world, character),
            color: world.get::<&AftikColor>(character).ok().map(|color| *color),
            direction: world
                .get::<&Direction>(character)
                .map(|direction| *direction)
                .unwrap_or_default(),
        }
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
