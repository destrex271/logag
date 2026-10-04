use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessResponse {
    pub user_input: String,
    pub agent_output: String,
    pub project_lane: String, // This is supposed to be a UUID.
}
