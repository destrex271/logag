use crate::shared_log::traits::{Event, EventFactory, EventType, SharedLog};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};

#[derive(Clone)]
pub struct EventAggregator {
    event_factory: EventFactory,
}

impl SharedLog for EventAggregator {
    fn append_event(&self, event: Box<dyn Event>) {
        println!(
            "{:?}, {:?}, {:?}",
            event.get_content(),
            event.get_event_type(),
            event.get_id()
        )
    }
    // fn consume_from_beginning(&self) -> impl Iterator<Item = LogEvent> + '_ {}
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct MCPEvent {
    event_type: EventType,
    content: String,
    timestamp: String,
    agent_notes: String,
}

#[tool_router(server_handler)]
impl EventAggregator {
    pub fn new() -> Self {
        EventAggregator {
            event_factory: EventFactory::new(),
        }
    }

    #[tool(description = "Register User/Agent/Thinking event")]
    pub fn add_event(
        &self,
        Parameters(MCPEvent {
            event_type,
            content,
            timestamp,
            agent_notes,
        }): Parameters<MCPEvent>,
    ) -> String {
        self.append_event(
            self.event_factory
                .create_log_event(content, timestamp, event_type),
        );
        println!("{:?}", agent_notes);
        "success".to_string()
    }
}
