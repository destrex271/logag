use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HarnessResponse {
    pub user_input: String,
    pub agent_output: String,
}
