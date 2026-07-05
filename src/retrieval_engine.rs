use std::sync::Arc;

use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    global_config::GlobalConfig,
    language_handler::service::LanguageService,
    shared_log::{
        agent_recorder::AgentRecorder, traits::SharedLog,
        user_embedding_model::SlimUserEmbeddingInput,
    },
};

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct ReadQuery {
    content: String,
}

#[derive(Clone)]
pub struct RetrievalEngine {
    shared_log: Arc<AgentRecorder>,
    lang_service: LanguageService,
}

#[tool_router(server_handler)]
impl RetrievalEngine {
    pub fn new(global_config: GlobalConfig) -> Self {
        let recorder = Arc::new(AgentRecorder::new());
        recorder.initialize(global_config.clone());
        RetrievalEngine {
            shared_log: recorder,
            lang_service: LanguageService::new(),
        }
    }

    #[tool(description = "Get latest agent output that was stored for similar user query.")]
    pub fn get_cached_agent_response(
        &self,
        Parameters(ReadQuery { content }): Parameters<ReadQuery>,
    ) -> String {
        let formatted_content = self.lang_service.format_content(&content);

        let user_event_ref: SlimUserEmbeddingInput = match self
            .shared_log
            .find_similar_user_event(formatted_content.clone())
        {
            Ok(response) => response,
            Err(err) => return format!("unable to find similar user input: {:?}", err),
        };

        let agent_response = self
            .shared_log
            .fetch_ai_response_for_user_event(user_event_ref.user_event_id);

        match agent_response {
            Ok(response) => {
                tracing::info!(
                    "Cache hit for user query {}: {}",
                    formatted_content.clone(),
                    response.get_content()
                );
                response.get_content()
            }
            Err(err) => {
                tracing::error!("error when fetching agent response {}", err);
                "no agent response exists".to_string()
            }
        }
    }
}
