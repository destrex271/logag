use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;

use crate::shared_log::log::LogEvent;
use crate::shared_log::user_embedding_model::SlimUserEmbeddingInput;

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub enum EventType {
    UserInput,
    AgentOutput,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::UserInput => write!(f, "user_input"),
            EventType::AgentOutput => write!(f, "agent_output"),
        }
    }
}

pub trait SharedLog {
    fn append_event(&self, event: Box<dyn Event>) -> uuid::Uuid;
    fn append_event_pair(&self, user_input: Box<dyn Event>, agent_output: Box<dyn Event>);
    fn find_similar_user_event(&self, user_input: Box<dyn Event>) -> SlimUserEmbeddingInput;
}

#[async_trait::async_trait]
pub trait Event: Send + Sync {
    fn get_event_type(&self) -> EventType;
    fn get_content(&self) -> String;
    fn get_id(&self) -> uuid::Uuid; // Only use uuid v7.
    fn get_timestamp(&self) -> isize;
}

pub trait EventReference: Send + Sync {
    fn get_source_id(&self) -> uuid::Uuid;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display_user_input() {
        assert_eq!(EventType::UserInput.to_string(), "user_input");
    }

    #[test]
    fn test_event_type_display_agent_output() {
        assert_eq!(EventType::AgentOutput.to_string(), "agent_output");
    }

    #[test]
    fn test_event_factory_creates_log_event() {
        let factory = EventFactory::new();
        let event = factory.create_log_event(
            "hello".to_string(),
            "1000000".to_string(),
            EventType::UserInput,
        );
        assert_eq!(event.get_content(), "hello");
        assert_eq!(event.get_timestamp(), 1000000);
    }
}
