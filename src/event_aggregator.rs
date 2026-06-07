use crate::global_config::GlobalConfig;
use crate::shared_log::traits::{Event, EventFactory, EventType, SharedLog};
use crate::storage::traits::{StorageBackendProvider, StorageEngine};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use tokio::sync::Mutex;

pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: AgentRecorder,
}

struct AgentRecorder {
    storage: std::sync::Arc<Mutex<Option<Box<dyn StorageEngine>>>>,
}

impl AgentRecorder {
    fn new() -> Self {
        AgentRecorder {
            storage: std::sync::Arc::new(Mutex::new(None)),
        }
    }

    fn initialize(&self, config: GlobalConfig) {
        let storage = self.storage.clone();
        tokio::spawn(async move {
            let engine = StorageBackendProvider::get_storage_backend(config).await;
            *storage.lock().await = Some(engine);
        });
    }
}

impl SharedLog for AgentRecorder {
    fn append_event(&self, event: Box<dyn Event>) {
        tracing::info!(
            content = %event.get_content(),
            event_type = %event.get_event_type(),
            id = %event.get_id(),
            "appended event"
        );

        let storage = self.storage.clone();
        tokio::spawn(async move {
            let guard = storage.lock().await;
            if let Some(ref engine) = *guard {
                let _ = engine.store_event(event.as_ref()).await;
            }
        });
    }
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
        EventAggregator::new(self.global_config.clone())
    }
}

#[tool_router(server_handler)]
impl EventAggregator {
    pub fn new(global_config: GlobalConfig) -> Self {
        let recorder = AgentRecorder::new();
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
        self.shared_log
            .append_event(self.event_factory.create_log_event(
                content,
                unix_epoch_timestamp,
                event_type,
            ));
        println!("{:?}", agent_notes);
        tracing::info!(agent_notes = %agent_notes, "add_event called");
        "success".to_string()
    }
}
