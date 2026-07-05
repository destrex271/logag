use std::str::FromStr;

use crate::shared_log::traits::{Event, EventReference, EventType};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use uuid::Timestamp;

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct LogContent {
    content: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogEvent {
    id: String,
    event_type: EventType,
    content: LogContent,
    timestamp: String,
}

impl Event for LogEvent {
    fn get_event_type(&self) -> EventType {
        self.event_type.clone()
    }

    fn get_content(&self) -> String {
        self.content.content.clone()
    }

    fn get_id(&self) -> uuid::Uuid {
        uuid::Uuid::from_str(self.id.as_str()).unwrap()
    }

    fn get_timestamp(&self) -> isize {
        self.timestamp.parse::<isize>().unwrap()
    }
}

impl LogContent {
    pub fn new(content: String) -> Self {
        Self { content }
    }
    pub fn get_content(&self) -> String {
        self.content.clone()
    }
}

impl LogEvent {
    pub fn new(event_type: EventType, content: String, timestamp: String) -> Self {
        let content = LogContent::new(content);
        let seconds = timestamp.parse::<isize>().unwrap();
        let id = uuid::Uuid::new_v7(Timestamp::from_unix_time(seconds as u64, 0, 0, 0));
        Self {
            id: id.to_string(),
            event_type,
            content,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_log::traits::{Event, EventType};

    use super::*;

    #[test]
    fn test_log_event_new_and_getters() {
        let event = LogEvent::new(
            EventType::UserInput,
            "test content".into(),
            "987654321".into(),
        );
        assert_eq!(event.get_event_type().to_string(), "user_input");
        assert_eq!(event.get_content(), "test content");
        assert_eq!(event.get_timestamp(), 987654321);
    }
}
