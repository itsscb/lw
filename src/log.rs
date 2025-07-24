use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    id: Uuid,
    content: String,
    created: DateTime<Local>,
    modified: DateTime<Local>,
}

impl Item {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, content: String) {
        self.content = content;
        self.modified = Local::now();
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }
}

impl Default for Item {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            content: String::new(),
            created: now,
            modified: now,
        }
    }
}

impl<T: AsRef<str>> From<T> for Item {
    fn from(value: T) -> Self {
        let mut item = Self::new();
        item.content = value.as_ref().to_owned();
        item
    }
}
