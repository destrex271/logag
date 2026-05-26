use crate::shared_log::traits::{Event, EventType};
use crate::storage::traits::BaseStorageEntity;
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use uuid::Timestamp;

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct LogContent {
    content: String,
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct LogEvent {
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
        let seconds = self.get_timestamp();
        let id = uuid::Uuid::new_v7(Timestamp::from_unix_time(seconds as u64, 0, 0, 0));
        return id;
    }

    fn get_timestamp(&self) -> isize {
        self.timestamp.parse::<isize>().unwrap()
    }
}

impl LogContent {
    fn new(content: String) -> Self {
        Self { content }
    }
}

impl LogEvent {
    pub fn new(event_type: EventType, content: String, timestamp: String) -> Self {
        let content = LogContent::new(content);
        Self {
            event_type: event_type,
            content: content,
            timestamp: timestamp,
        }
    }
}

impl BaseStorageEntity for LogEvent{
    fn construct(raw_data: std::iter::Map<String, Box<dyn std::any::Any>>) -> Self {
        LogEvent::new(raw_data["event_type"], raw_data["content"], raw_data["timestamp"])
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
