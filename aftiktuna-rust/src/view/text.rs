use crate::OneOrTwo;
use crate::asset::GameAssets;
use crate::core::name::{self, NameIdData};
use hecs::Entity;

#[derive(Clone)]
pub(crate) enum Message {
    Combinable(CombinableMsgType, Vec<NameIdData>),
    String(String),
}

impl Message {
    fn try_combine(self, other: Self) -> OneOrTwo<Self> {
        match (self, other) {
            (Self::Combinable(type_1, entities_1), Self::Combinable(type_2, entities_2)) => {
                if let Some(type_3) = type_1.try_combine(&type_2) {
                    let mut entities = entities_1;
                    entities.extend(entities_2);
                    OneOrTwo::One(Self::Combinable(type_3, entities))
                } else {
                    OneOrTwo::Two(
                        Self::Combinable(type_1, entities_1),
                        Self::Combinable(type_2, entities_2),
                    )
                }
            }
            (msg1, msg2) => OneOrTwo::Two(msg1, msg2),
        }
    }

    fn into_text(self, assets: &GameAssets) -> String {
        match self {
            Message::Combinable(msg_type, entities) => msg_type.into_text(entities, assets),
            Message::String(text) => text,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum CombinableMsgType {
    Noise,
    EnterDoor(Entity, NameIdData),
    EnterPath(Entity, NameIdData),
    Arrive(Entity),
    PickUp(NameIdData),
    Threatening,
    Attacking,
}

impl CombinableMsgType {
    pub fn message(self, entity_name: NameIdData) -> Message {
        Message::Combinable(self, vec![entity_name])
    }

    fn try_combine(&self, other: &Self) -> Option<Self> {
        if self == other {
            Some(self.clone())
        } else if let Self::EnterDoor(path1, _) | Self::EnterPath(path1, _) = self
            && let Self::Arrive(path2) = other
            && path1 == path2
        {
            Some(self.clone())
        } else {
            None
        }
    }

    fn into_text(self, entities: Vec<NameIdData>, assets: &GameAssets) -> String {
        use CombinableMsgType::*;
        match self {
            Noise => format!(
                "Something is making noise in the direction of {the_paths}.",
                the_paths = join_elements(unique(
                    entities
                        .into_iter()
                        .map(|name| name.lookup(assets).definite())
                ))
            ),
            EnterDoor(_, door_name) => format!(
                "{the_characters} entered {the_door} into a new area.",
                the_characters = capitalize(join_elements(
                    entities
                        .into_iter()
                        .map(|name| name.lookup(assets).definite())
                        .collect()
                )),
                the_door = door_name.lookup(assets).definite(),
            ),
            EnterPath(_, path_name) => format!(
                "{the_characters} followed {the_path} to a new area.",
                the_characters = capitalize(join_elements(
                    entities
                        .into_iter()
                        .map(|name| name.lookup(assets).definite())
                        .collect()
                )),
                the_path = path_name.lookup(assets).definite(),
            ),
            Arrive(_) => format!(
                "{the_characters} arrived from a nearby area.",
                the_characters = capitalize(join_elements(name::names_with_counts(
                    entities,
                    name::ArticleKind::A,
                    name::CountFormat::Text,
                    assets,
                )))
            ),
            PickUp(performer_name) => format!(
                "{the_performer} picked up {the_items}.",
                the_performer = capitalize(performer_name.lookup(assets).definite()),
                the_items = join_elements(name::names_with_counts(
                    entities,
                    name::ArticleKind::The,
                    name::CountFormat::Text,
                    assets,
                ))
            ),
            Threatening => {
                if let [entity] = &entities[..] {
                    format!(
                        "{the_creature} makes a threatening pose.",
                        the_creature = capitalize(entity.clone().lookup(assets).definite()),
                    )
                } else {
                    format!(
                        "{the_creatures} make threatening poses.",
                        the_creatures = capitalize(join_elements(name::names_with_counts(
                            entities,
                            name::ArticleKind::The,
                            name::CountFormat::Text,
                            assets,
                        ))),
                    )
                }
            }
            Attacking => {
                if let [entity] = &entities[..] {
                    format!(
                        "{the_creature} moves in to attack.",
                        the_creature = capitalize(entity.clone().lookup(assets).definite()),
                    )
                } else {
                    format!(
                        "{the_creatures} move in to attack.",
                        the_creatures = capitalize(join_elements(name::names_with_counts(
                            entities,
                            name::ArticleKind::The,
                            name::CountFormat::Text,
                            assets,
                        ))),
                    )
                }
            }
        }
    }
}

pub(crate) trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for Message {
    fn into_message(self) -> Message {
        self
    }
}

impl IntoMessage for &Message {
    fn into_message(self) -> Message {
        self.clone()
    }
}

impl IntoMessage for String {
    fn into_message(self) -> Message {
        if self.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
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
pub(crate) struct Messages(Vec<Message>);

impl Messages {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, message: impl IntoMessage) {
        self.0.push(message.into_message());
    }

    pub fn into_text(self, assets: &GameAssets) -> Vec<String> {
        let combined_messages = crate::try_combine_adjacent(self.0, Message::try_combine);
        combined_messages
            .into_iter()
            .map(|message| message.into_text(assets))
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

fn unique(elements: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut unique_element_list = Vec::new();
    for element in elements {
        if !unique_element_list.contains(&element) {
            unique_element_list.push(element);
        }
    }
    unique_element_list
}
