use std::{collections::HashMap, fs::File, io::Read, path::Path};

use crate::global_config::GlobalConfig;

pub enum LLMWorkerErrors {
    ProviderError(String),
}

pub enum ConsolidationError {
    LLMFailure(String),
    UnableToReadInstructions(String),
    NotImplemented(String),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ProcessTier {
    Tier3,
}

pub struct ConsolidateResponseSlice {
    user_prompt: String,
    agent_response: String,
}

pub struct ConsolidatedResponse {
    response: Vec<ConsolidateResponseSlice>,
}

pub struct AgentPromptPayload {
    timestamp: usize,
    content: String,
}

pub struct ConsolidationPayload {
    user_prompt: String,
    agent_prompts: AgentPromptPayload,
}

pub struct LLMPayload {
    system_prompt: String,
    user_prompt: String,
}

#[async_trait::async_trait]
pub trait LLMWorker: Send + Sync {
    async fn send_request(&self, body: String) -> Result<String, LLMWorkerErrors>;
}

pub struct MemoryConsolidator {
    worker: Box<dyn LLMWorker>,
    tier_to_file_map: HashMap<ProcessTier, String>,
}

impl MemoryConsolidator {
    fn new(worker: Box<dyn LLMWorker>, global_config: &GlobalConfig) -> Self {
        let mut map: HashMap<ProcessTier, String> = HashMap::new();
        map.insert(
            ProcessTier::Tier3,
            String::from(global_config.fetch_bin_root() + "static/tier3_prompt.md"),
        );
        MemoryConsolidator {
            worker: worker,
            tier_to_file_map: map,
        }
    }

    fn read_base_instruction_file(
        &self,
        tier_level: &ProcessTier,
    ) -> Result<String, ConsolidationError> {
        let path = self.tier_to_file_map.get(&tier_level.clone()).unwrap();
        match File::open(Path::new(path.clone().as_str())) {
            Err(err) => Err(ConsolidationError::UnableToReadInstructions(format!(
                "Unable to read: {}; {}",
                path.clone(),
                err.to_string()
            ))),
            Ok(mut fl) => {
                let mut data: String = String::new();
                match fl.read_to_string(&mut data) {
                    Err(err) => Err(ConsolidationError::UnableToReadInstructions(format!(
                        "Error while reading file: {}",
                        err.to_string()
                    ))),
                    Ok(_) => Ok(data),
                }
            }
        }
    }

    fn create_memory_consolidation_request(
        &self,
        tier_level: &ProcessTier,
        consolidation_request: ConsolidationPayload,
    ) -> LLMPayload {
        let base_instructions = self.read_base_instruction_file(&tier_level);
        // TODO(destrex271): Construct actual payload here.
    }

    async fn send_memory_consolidation_request(
        &self,
        payload: LLMPayload,
    ) -> Result<ConsolidatedResponse, ConsolidationError> {
        Err(ConsolidationError::NotImplemented(
            "Not implemented".to_string(),
        ))
    }
}
