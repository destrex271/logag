use crate::shared_log::traits::{Event, EventType};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct LogContent {
    content: String,
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct LogEvent {
    event_type: EventType,
    content: LogContent,
    timestamp: String,
    id: String,
}

impl Event for LogEvent {
    fn get_event_type(&self) -> EventType {
        self.event_type.clone()
    }

    fn get_content(&self) -> String {
        self.content.content.clone()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_timestamp(&self) -> String {
        self.timestamp.clone()
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
        let uuid = Uuid::new_v4().to_string();

        Self {
            event_type: event_type,
            content: content,
            timestamp: timestamp,
            id: uuid.clone(),
        }
    }
}
