use crate::global_config::GlobalConfig;
use crate::shared_log::traits::{Event, EventFactory, EventType, SharedLog};
use crate::storage::traits::{StorageBackendProvider, StorageEngine};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};

pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: AgentRecorder, // Split into registry pattern if different types of shared logs pop
                               // up.
}

struct AgentRecorder {
    storage: Box<dyn StorageEngine>,
}

impl AgentRecorder {
    fn new(config: GlobalConfig) -> Self {
        let storage_engine = StorageBackendProvider::get_storage_backend(config);
        AgentRecorder {
            storage: storage_engine,
        }
    }
}

impl SharedLog for AgentRecorder {
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
    unix_epoch_timestamp: String, // Expected format is unix epoch time.
    agent_notes: String,
}

impl Clone for EventAggregator {
    fn clone(&self) -> Self {
        EventAggregator::new(self.global_config.clone())
    }
}

#[tool_router(server_handler)]
impl EventAggregator {
    pub fn new(global_config: GlobalConfig) -> Self {
        EventAggregator {
            event_factory: EventFactory::new(),
            shared_log: AgentRecorder::new(global_config.clone()),
            global_config: global_config,
        }
    }

    #[tool(description = "Register User/Agent/Thinking event")]
    pub fn add_event(
        &self,
        Parameters(MCPEvent {
            event_type,
            content,
            unix_epoch_timestamp,
            agent_notes,
        }): Parameters<MCPEvent>,
    ) -> String {
        self.shared_log
            .append_event(self.event_factory.create_log_event(
                content,
                unix_epoch_timestamp,
                event_type,
            ));
        println!("{:?}", agent_notes);
        "success".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_config::GlobalConfig;
    use crate::storage::traits::StorageBackend;

    #[tokio::test]
    async fn test_event_aggregator_new() {
        let config = GlobalConfig {
            storage_backend: StorageBackend::Postgres,
            database_connection_string: "postgres://localhost:5432/test".into(),
        };
        let aggregator = EventAggregator::new(config);
        assert_eq!(
            aggregator.global_config.database_connection_string,
            "postgres://localhost:5432/test"
        );
    }
}
