pub enum LLMWorkerErrors {
    ProviderError(String),
}

pub enum ConsolidationError{
    LLMFailure(String),
    NotImplemented(String),
}

pub struct ConsolidateResponseSlice{
    user_prompt: String,
    agent_response: String,
}

pub struct ConsolidatedResponse{
    response: Vec<ConsolidateResponseSlice>,
}

pub struct AgentPromptPayload{
    timestamp: usize,
    content: String,
}

pub struct ConsolidationPayload{
    user_prompt: String,
    agent_prompts: AgentPromptPayload,
}

#[async_trait::async_trait]
pub trait LLMWorker: Send + Sync{
    async fn send_request(&self, body: String) -> Result<String, LLMWorkerErrors>;
}

pub struct MemoryConsolidator{
    worker: Box<dyn LLMWorker>
}

impl MemoryConsolidator {
    async fn send_tier3_memory_consolidation_request(
        &self,
        _payload: ConsolidationPayload
    ) -> Result<ConsolidatedResponse, ConsolidationError>{ 
        Err(ConsolidationError::NotImplemented("Not implemented".to_string()))
    }
}

