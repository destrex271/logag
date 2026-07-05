use crate::global_config::GlobalConfig;
use crate::shared_log::agent_recorder::AgentRecorder;
use crate::shared_log::traits::{EventFactory, EventType, SharedLog};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};

pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct MCPEvent {
    event_type: EventType,
    content: String,
    unix_epoch_timestamp: String,
    agent_notes: String,
}

impl Clone for EventAggregator {
    fn clone(&self) -> Self {
        EventAggregator {
            event_factory: EventFactory::new(),
            global_config: self.global_config.clone(),
            shared_log: self.shared_log.clone(),
        }
    }
}

#[tool_router(server_handler)]
impl EventAggregator {
    pub fn new(global_config: GlobalConfig) -> Self {
        let recorder = std::sync::Arc::new(AgentRecorder::new());
        recorder.initialize(global_config.clone());
        EventAggregator {
            event_factory: EventFactory::new(),
            shared_log: recorder,
            global_config,
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
        let formatted_content = self.format_content(&content);
        self.shared_log
            .append_event(self.event_factory.create_log_event(
                formatted_content,
                unix_epoch_timestamp,
                event_type,
            ));
        tracing::info!("{:?}", agent_notes);
        tracing::info!(agent_notes = %agent_notes, "add_event called");
        "success".to_string()
    }

    pub fn add_event_pair_from_raw_stream(
        &self,
        user_content: String,
        agent_content: String,
        unix_epoch_timestamp: String,
    ) -> String {
        self.shared_log.append_event_pair(
            self.event_factory.create_log_event(
                self.format_content(&user_content),
                unix_epoch_timestamp.clone(),
                EventType::UserInput,
            ),
            self.event_factory.create_log_event(
                self.format_content(&agent_content),
                unix_epoch_timestamp,
                EventType::AgentOutput,
            ),
        );
        "success".to_string()
    }

    fn format_content(&self, content: &str) -> String{
        // TODO(destrex271): Use NLP techniques like stemming etc to improve data quality.
        let new_content = content.trim().to_lowercase();
        new_content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_log::agent_recorder::test_utils::MockStorageEngine;
    use crate::shared_log::traits::EventType;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_event_aggregator_add_event_with_storage() {
        let store_count = std::sync::Arc::new(AtomicUsize::new(0));
        let engine = Box::new(MockStorageEngine {
            store_count: store_count.clone(),
        });
        let recorder = AgentRecorder::new_with_storage(engine);
        let factory = EventFactory::new();
        let global_config = GlobalConfig {
            storage_backend: crate::storage::traits::StorageBackend::Postgres,
            database_connection_string: "postgres://localhost:5432/test".into(),
        };

        let aggregator = EventAggregator {
            event_factory: factory,
            global_config,
            shared_log: std::sync::Arc::new(recorder),
        };

        let mcp_event = MCPEvent {
            event_type: EventType::AgentOutput,
            content: "test output".into(),
            unix_epoch_timestamp: "2000000".into(),
            agent_notes: "notes".into(),
        };

        let result = aggregator.add_event(Parameters(mcp_event));
        assert_eq!(result, "success");

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(store_count.load(Ordering::SeqCst), 1);
    }
}
