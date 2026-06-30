use crate::embeddings::fastembed::FastEmbeddingService;
use crate::embeddings::traits::EmbeddingsService;
use crate::storage::traits::{StorageBackendProvider, StorageEngine};
use crate::shared_log::traits::{Event, SharedLog};
use crate::global_config::GlobalConfig;
use tokio::sync::OnceCell;

pub struct AgentRecorder {
    storage: std::sync::Arc<OnceCell<Box<dyn StorageEngine>>>,
    embedding_model: std::sync::Arc<OnceCell<std::sync::Mutex<Box<dyn EmbeddingsService>>>>,
}

impl AgentRecorder {
    pub fn new() -> Self {
        AgentRecorder {
            storage: std::sync::Arc::new(OnceCell::new()),
            embedding_model: std::sync::Arc::new(OnceCell::new()),
        }
    }

    pub fn initialize(&self, config: GlobalConfig) {
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
        let embeddings: Option<Vec<f32>> = self.embedding_model.get().and_then(
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

#[cfg(test)]
impl AgentRecorder {
    pub(crate) fn new_with_storage(engine: Box<dyn StorageEngine>) -> Self {
        let storage = std::sync::Arc::new(tokio::sync::OnceCell::new());
        let _ = storage.set(engine);
        let model_cell = std::sync::Arc::new(tokio::sync::OnceCell::new());
        let _ = model_cell.set(std::sync::Mutex::new(
            Box::new(FastEmbeddingService::new().unwrap()) as Box<dyn EmbeddingsService>,
        ));
        AgentRecorder {
            storage,
            embedding_model: model_cell,
        }
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;
    use crate::shared_log::traits::Event;
    use crate::storage::traits::StorageEngineErrors;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub(crate) struct MockStorageEngine {
        pub(crate) store_count: std::sync::Arc<AtomicUsize>,
    }

    #[async_trait]
    impl StorageEngine for MockStorageEngine {
        async fn load_storage(_config: crate::global_config::GlobalConfig) -> Self {
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
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockStorageEngine;
    use super::*;
    use crate::shared_log::traits::{EventFactory, EventType};
    use std::sync::atomic::{AtomicUsize, Ordering};

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
}
