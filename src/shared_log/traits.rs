use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;

use crate::shared_log::log::LogEvent;

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub enum EventType {
    UserInput,
    AgentOutput,
}

pub trait SharedLog {
    fn append_event(&self, event: Box<dyn Event>);
}

pub trait EventContent {
    fn get_content(&self) -> &str;
}

pub trait Event {
    fn get_event_type(&self) -> EventType;
    fn get_content(&self) -> String;
    fn get_id(&self) -> String;
    fn get_timestamp(&self) -> String;
}

#[derive(Clone)]
pub struct EventFactory;

impl EventFactory {
    pub fn new() -> Self {
        EventFactory {}
    }

    pub fn create_log_event(
        &self,
        content: String,
        timestamp: String,
        event_type: EventType,
    ) -> Box<dyn Event> {
        Box::new(LogEvent::new(event_type, content, timestamp))
    }
}
