use crate::core::name::{self, NameData};
use crate::OneOrTwo;

#[derive(Debug)]
pub enum Message {
    Combinable(CombinableMsgType, Vec<NameData>),
    String(String),
}

impl Message {
    fn try_combine(self, other: Self) -> OneOrTwo<Self> {
        match (self, other) {
            (Self::Combinable(type_1, mut entities_1), Self::Combinable(type_2, entities_2))
                if type_1 == type_2 =>
            {
                entities_1.extend(entities_2);
                OneOrTwo::One(Self::Combinable(type_1, entities_1))
            }
            (msg1, msg2) => OneOrTwo::Two(msg1, msg2),
        }
    }

    fn into_text(self) -> String {
        match self {
            Message::Combinable(msg_type, entities) => msg_type.into_text(entities),
            Message::String(text) => text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombinableMsgType {
    EnterDoor,
    EnterPath,
    PickUp(NameData),
    Threatening,
    Attacking,
}

impl CombinableMsgType {
    pub fn message(self, entity_name: NameData) -> Message {
        Message::Combinable(self, vec![entity_name])
    }

    fn into_text(self, entities: Vec<NameData>) -> String {
        use CombinableMsgType::*;
        match self {
            EnterDoor => format!(
                "{the_characters} entered the door into a new area.",
                the_characters = capitalize(join_elements(
                    entities.into_iter().map(|name| name.definite()).collect()
                ))
            ),
            EnterPath => format!(
                "{the_characters} followed the path to a new area.",
                the_characters = capitalize(join_elements(
                    entities.into_iter().map(|name| name.definite()).collect()
                ))
            ),
            PickUp(performer_name) => format!(
                "{the_performer} picked up {the_items}.",
                the_performer = capitalize(performer_name.definite()),
                the_items = join_elements(name::names_with_counts(
                    entities,
                    name::Article::The,
                    name::CountFormat::Text
                ))
            ),
            Threatening => {
                if let [entity] = &entities[..] {
                    format!(
                        "{the_creature} makes a threatening pose.",
                        the_creature = capitalize(entity.definite()),
                    )
                } else {
                    format!(
                        "{the_creatures} make threatening poses.",
                        the_creatures = capitalize(join_elements(name::names_with_counts(
                            entities,
                            name::Article::The,
                            name::CountFormat::Text,
                        ))),
                    )
                }
            }
            Attacking => {
                if let [entity] = &entities[..] {
                    format!(
                        "{the_creature} moves in to attack.",
                        the_creature = capitalize(entity.definite()),
                    )
                } else {
                    format!(
                        "{the_creatures} move in to attack.",
                        the_creatures = capitalize(join_elements(name::names_with_counts(
                            entities,
                            name::Article::The,
                            name::CountFormat::Text,
                        ))),
                    )
                }
            }
        }
    }
}

pub trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for Message {
    fn into_message(self) -> Message {
        self
    }
}

impl IntoMessage for String {
    fn into_message(self) -> Message {
        if self
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_lowercase())
        {
            Message::String(capitalize(self))
        } else {
            Message::String(self)
        }
    }
}

impl IntoMessage for &str {
    fn into_message(self) -> Message {
        Message::String(capitalize(self))
    }
}

#[derive(Default)]
pub struct Messages(Vec<Message>);

impl Messages {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add(&mut self, message: impl IntoMessage) {
        self.0.push(message.into_message());
    }

    pub fn into_text(self) -> Vec<String> {
        let combined_messages = crate::try_combine_adjacent(self.0, Message::try_combine);
        combined_messages
            .into_iter()
            .map(Message::into_text)
            .collect()
    }
}

impl<T: IntoMessage> From<T> for Messages {
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

pub fn join_elements(mut elements: Vec<String>) -> String {
    let Some(last_element) = elements.pop() else {
        return String::new();
    };
    if elements.is_empty() {
        last_element
    } else {
        format!(
            "{first_elements} and {last_element}",
            first_elements = elements.join(", ")
        )
    }
}
