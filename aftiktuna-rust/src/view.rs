use crate::action::trade;
use crate::action::trade::{PricedItem, Shopkeeper};
use crate::area::Choice;
use crate::game_loop::StopType;
use hecs::{Entity, World};
pub use location::{AftikColor, ObjectRenderData, RenderData, TextureType};
pub use name::{as_grouped_text_list, NounData};
use serde::{Deserialize, Serialize};
pub use status::print_full_status;
use std::mem::take;

mod location;
mod name;
mod status;

#[derive(Default)]
pub struct Messages(Vec<String>);

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayInfo {
    symbol: char,
    pub texture_type: TextureType,
    weight: u32,
}

pub type StatusCache = status::Cache;

pub type NameData = name::Data;

#[derive(Default)]
pub struct Buffer {
    pub messages: Messages,
    captured_frames: Vec<Frame>,
}

impl Buffer {
    pub fn capture_view(&mut self, world: &World, character: Entity, cache: &mut StatusCache) {
        let frame = if let Some(shopkeeper) = trade::get_shop_info(world, character) {
            shop_frame(&shopkeeper, self, world, character, cache)
        } else {
            area_view_frame(self, world, character, cache)
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

    pub fn push_ending_frame(&mut self, world: &World, character: Entity, stop_type: StopType) {
        self.push_frame(Frame::Ending {
            stop_type,
            render_data: location::prepare_render_data(world, character),
        })
    }

    pub fn into_frames(self) -> Vec<Frame> {
        self.captured_frames
    }
}

pub enum Frame {
    Introduction,
    AreaView {
        messages: Messages,
        render_data: RenderData,
    },
    StoreView {
        view: StoreView,
        messages: Messages,
    },
    LocationChoice(Choice),
    Ending {
        stop_type: StopType,
        render_data: RenderData,
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
                text.extend(location::area_view_messages(render_data).0);

                if !messages.0.is_empty() {
                    text.push(String::default());

                    text.extend(messages.0.clone());
                }
            }
            Frame::StoreView { view, messages, .. } => {
                text.push("--------------------".to_string());
                text.extend(view.messages().0);

                if !messages.0.is_empty() {
                    text.push(String::default());

                    text.extend(messages.0.clone());
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
            Frame::AreaView { messages, .. } | Frame::StoreView { messages, .. } => {
                !messages.0.is_empty()
            }
            Frame::Introduction | Frame::LocationChoice(_) | Frame::Ending { .. } => true,
        }
    }

    pub fn get_messages(&self) -> Vec<String> {
        match self {
            Frame::Introduction => intro_messages(),
            Frame::AreaView { messages, .. } => messages.0.clone(),
            Frame::StoreView { messages, .. } => messages.0.clone(),
            Frame::LocationChoice(choice) => choice.present().0,
            Frame::Ending { stop_type, .. } => stop_type.messages().0,
        }
    }
}

fn intro_messages() -> Vec<String> {
    vec!["Welcome to Aftiktuna!".to_string(),"Your goal is to lead a group of aftiks on their journey through space to find the fabled Fortuna chest, which is said to contain the item that the finder desires the most.".to_string()]
}

#[derive(Serialize, Deserialize)]
pub struct StoreView {
    items: Vec<PricedItem>,
    points: i32,
}

impl StoreView {
    pub fn messages(&self) -> Messages {
        let mut messages = Messages::default();
        let items = self
            .items
            .iter()
            .map(|priced| (capitalize(priced.item.noun_data().singular()), priced.price))
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

fn shop_frame(
    shopkeeper: &Shopkeeper,
    buffer: &mut Buffer,
    world: &World,
    character: Entity,
    cache: &mut StatusCache,
) -> Frame {
    // Use the cache in shop view before status messages so that points aren't shown in status messages too
    let points = status::fetch_points(world, character, cache);
    let items = shopkeeper.0.clone();
    Frame::StoreView {
        view: StoreView { items, points },
        messages: buffer.pop_messages(world, character, cache),
    }
}

fn area_view_frame(
    buffer: &mut Buffer,
    world: &World,
    character: Entity,
    cache: &mut StatusCache,
) -> Frame {
    Frame::AreaView {
        messages: buffer.pop_messages(world, character, cache),
        render_data: location::prepare_render_data(world, character),
    }
}

impl DisplayInfo {
    pub fn new(symbol: char, texture_type: TextureType, weight: u32) -> Self {
        DisplayInfo {
            symbol,
            texture_type,
            weight,
        }
    }
}

pub fn capitalize(text: impl AsRef<str>) -> String {
    let mut chars = text.as_ref().chars();
    match chars.next() {
        None => String::new(),
        Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn name_display_info(texture_type: TextureType, name: &str) -> DisplayInfo {
    DisplayInfo::new(name.chars().next().unwrap(), texture_type, 10)
}
