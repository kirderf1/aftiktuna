use crate::OneOrTwo;

#[derive(Debug)]
pub enum Message {
    Combinable(CombinableMsgType, Vec<String>),
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
            Message::Combinable(msg_type, entities) => msg_type.into_text(join_elements(entities)),
            Message::String(text) => text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombinableMsgType {
    EnterDoor,
    EnterPath,
}

impl CombinableMsgType {
    pub fn message(self, entity: impl Into<String>) -> Message {
        Message::Combinable(self, vec![entity.into()])
    }

    fn into_text(self, entities: String) -> String {
        use CombinableMsgType::*;
        match self {
            EnterDoor => capitalize(format!("{entities} entered the door into a new area.")),
            EnterPath => capitalize(format!("{entities} followed the path to a new area.")),
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
