use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HarnessResponse {
    pub userInput: String,
    pub agentOutput: String,
}
