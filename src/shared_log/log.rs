use crate::shared_log::traits::{Event, EventType};
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
