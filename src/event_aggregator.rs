use crate::embeddings::fastembed::FastEmbeddingService;
use crate::embeddings::traits::EmbeddingsService;
use crate::global_config::GlobalConfig;
use crate::shared_log::traits::{Event, EventFactory, EventType, SharedLog};
use crate::storage::traits::{StorageBackendProvider, StorageEngine};
use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use tokio::sync::OnceCell;

pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
}

struct AgentRecorder {
    storage: std::sync::Arc<OnceCell<Box<dyn StorageEngine>>>,
    embedding_model: std::sync::Arc<OnceCell<std::sync::Mutex<Box<dyn EmbeddingsService>>>>,
}

impl AgentRecorder {
    fn new() -> Self {
        AgentRecorder {
            storage: std::sync::Arc::new(OnceCell::new()),
            embedding_model: std::sync::Arc::new(OnceCell::new()),
        }
    }

    fn initialize(&self, config: GlobalConfig) {
        // TODO(destrex271): Find a better way to do this.
        let storage = self.storage.clone();
        tokio::spawn(async move {
            let engine = StorageBackendProvider::get_storage_backend(config).await;
            let _ = storage.set(engine);
        });

        let model = FastEmbeddingService::new().unwrap();
        let _ = self.embedding_model.set(std::sync::Mutex::new(Box::new(model)));
    }
}

impl SharedLog for AgentRecorder {
    fn append_event(&self, event: Box<dyn Event>) -> uuid::Uuid {
        let id: uuid::Uuid = event.get_id();

        tracing::info!(
            content = %event.get_content(),
            event_type = %event.get_event_type(),
            id = %event.get_id(),
            "appended standalone event to log"
        );

        let storage = self.storage.clone();
        let handle = tokio::spawn(async move {
            if let Some(engine) = storage.get() {
                let _ = engine.store_event(event.as_ref()).await;
            }
        });
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(handle).unwrap();
        });

        tracing::info!("appeneded event");
        return id
    }

    fn append_event_pair(&self, user_input: Box<dyn Event>, agent_output: Box<dyn Event>) {
        let user_content = user_input.get_content().clone();
        let user_event_id = self.append_event(user_input);
        let agent_event_id = self.append_event(agent_output);

        // Generate Embeddings for user.
        tracing::info_span!("generating_embeddings");
        let mut embeddings: Option<Vec<f32>> = self.embedding_model.get().and_then(
            |model| {
                model.lock().ok().and_then(
                    |mut m| {
                        m.generate_embeddings(
                            vec![user_content]
                        )
                            .ok()
                            .and_then(|v| v.into_iter().next())
                    }
                )
            }
        );
        tracing::info_span!("generated embeddings");

        let storage = self.storage.clone();
        tokio::spawn(async move{
            tracing::info!("inserting user embeddings for event id: {}", user_event_id);
            // Store embeddings.
            if let Some(embed) = embeddings{
                if let Some(engine) = storage.get(){
                    let _ = engine.store_user_input_embedding(user_event_id, &embed).await;
                }
            }
            tracing::info!("inserted user embeddings for event id: {}", user_event_id);

            // Store agent response.
            tracing::info!("inserting agent output cache for event id: {} for user query {}", agent_event_id, user_event_id);
            if let Some(engine) = storage.get() {
                let  _ = engine.store_cached_agent_response(agent_event_id, user_event_id).await;
            }
            tracing::info!("inserted agent output.")
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
        self.shared_log
            .append_event(self.event_factory.create_log_event(
                content,
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
                user_content,
                unix_epoch_timestamp.clone(),
                EventType::UserInput,
            ),
            self.event_factory.create_log_event(
                agent_content,
                unix_epoch_timestamp,
                EventType::AgentOutput,
            ),
        );
        "success".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_log::traits::{Event, EventType};
    use crate::storage::traits::{StorageEngine, StorageEngineErrors};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockStorageEngine {
        store_count: std::sync::Arc<AtomicUsize>,
    }

    #[async_trait]
    impl StorageEngine for MockStorageEngine {
        async fn load_storage(_config: GlobalConfig) -> Self {
            MockStorageEngine {
                store_count: std::sync::Arc::new(AtomicUsize::new(0)),
            }
        }

        async fn store_event(&self, _event: &dyn Event) -> Result<(), StorageEngineErrors> {
            self.store_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn store_user_input_embedding(
            &self,
            _user_event_id: uuid::Uuid,
            _embedding: &[f32],
        ) -> Result<(), StorageEngineErrors> {
            Ok(())
        }

        async fn store_cached_agent_response(
            &self,
            _log_event_id: uuid::Uuid,
            _user_query_id: uuid::Uuid,
        ) -> Result<(), StorageEngineErrors> {
            Ok(())
        }

        async fn get_events<T, F>(
            &self,
            _from_timestamp: isize,
            _to_timestamp: isize,
            _factory_fn: F,
        ) -> Result<Vec<T>, StorageEngineErrors>
        where
            T: Event,
            F: Fn(uuid::Uuid, String, isize, String) -> T + Send,
            Self: Sized,
        {
            Ok(Vec::new())
        }
    }

    #[cfg(test)]
    impl AgentRecorder {
        fn new_with_storage(engine: Box<dyn StorageEngine>) -> Self {
            let storage = std::sync::Arc::new(OnceCell::new());
            let _ = storage.set(engine);
            let model_cell = std::sync::Arc::new(OnceCell::new());
            let _ = model_cell.set(std::sync::Mutex::new(
                Box::new(FastEmbeddingService::new().unwrap()) as Box<dyn EmbeddingsService>,
            ));
            AgentRecorder {
                storage,
                embedding_model: model_cell,
            }
        }
    }

    #[test]
    fn test_agent_recorder_new_creates_empty_storage() {
        let recorder = AgentRecorder::new();
        assert!(recorder.storage.get().is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_recorder_append_event_no_storage() {
        let recorder = AgentRecorder::new();
        let event = EventFactory::new().create_log_event(
            "test".into(),
            "1000000".into(),
            EventType::UserInput,
        );
        recorder.append_event(event);
        // Give spawned task time to attempt (and skip) the store
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_recorder_append_event_with_storage() {
        let store_count = std::sync::Arc::new(AtomicUsize::new(0));
        let engine = Box::new(MockStorageEngine {
            store_count: store_count.clone(),
        });
        let recorder = AgentRecorder::new_with_storage(engine);

        let event = EventFactory::new().create_log_event(
            "test".into(),
            "1000000".into(),
            EventType::UserInput,
        );
        recorder.append_event(event);

        // Give the spawned task time to execute
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(store_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_recorder_append_event_multiple() {
        let store_count = std::sync::Arc::new(AtomicUsize::new(0));
        let engine = Box::new(MockStorageEngine {
            store_count: store_count.clone(),
        });
        let recorder = AgentRecorder::new_with_storage(engine);

        for i in 0..5 {
            let event = EventFactory::new().create_log_event(
                format!("event-{}", i),
                format!("{}", 1000000 + i),
                EventType::UserInput,
            );
            recorder.append_event(event);
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert_eq!(store_count.load(Ordering::SeqCst), 5);
    }

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
