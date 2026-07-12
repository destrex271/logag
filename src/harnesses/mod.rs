use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessResponse {
    pub user_input: String,
    pub agent_output: String,
}
