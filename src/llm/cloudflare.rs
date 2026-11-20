// Module to house cloudflare workers ai code.

use crate::llm::workers::{LLMWorker, LLMWorkerErrors};

const CF_ACCOUNT_ID: Option<&str> = option_env!("CLOUDFLARE_ACCOUNT_ID");
const CF_API_KEY: Option<&str> = option_env!("CLOUDFLARE_WORKERS_AI_API_KEY");
const BASE_URL: &str = "https://api.cloudflare.com/client/v4/accounts/";
const POST_URL: &str = "/ai/run/";

pub struct CloudflareWorker {
    model_identifier: String,
    http_client: reqwest::Client,
    cf_account_id: String,
    cf_api_key: String,
}

#[derive(serde::Serialize)]
struct CFWorkerAIRequest {
    prompt: String,
}

#[derive(serde::Deserialize)]
struct CFWorkerAIResponse {
    result: CFWorkersAIResponseSnippet,
    success: bool,
    errors: Vec<String>,
    messages: Vec<String>,
}

#[derive(serde::Deserialize)]
struct CFWorkersAIResponseSnippet {
    response: String,
}

impl CloudflareWorker {
    pub fn new(model_identifier: String) -> Self {
        let acc_id = match CF_ACCOUNT_ID {
            Some(val) => val,
            None => {
                tracing::error!("Cannot use CF Account Id; Unable to use CF Workers AI");
                panic!("Cannot use CF Account Id; Unable to use CF Workers AI")
            }
        };
        let api_key = match CF_API_KEY {
            Some(val) => val,
            None => {
                tracing::error!("Cannot use CF API Key; Unable to use CF Workers AI");
                panic!("Cannot use CF API Key; Unable to use CF Workers AI")
            }
        };
        CloudflareWorker {
            model_identifier,
            http_client: reqwest::Client::new(),
            cf_account_id: acc_id.to_string(),
            cf_api_key: api_key.to_string(),
        }
    }

    pub async fn send_request(&self, body: String) -> Result<String, LLMWorkerErrors> {
        let url = String::from(BASE_URL)
            + self.cf_account_id.as_str()
            + POST_URL
            + self.model_identifier.as_str();

        let response = self
            .http_client
            .post(url)
            .bearer_auth(self.cf_api_key.clone())
            .body(body)
            .send()
            .await
            .map_err(|err| LLMWorkerErrors::ProviderError(err.to_string()))?;

        let response: CFWorkerAIResponse = response
            .error_for_status()
            .map_err(|err| LLMWorkerErrors::ProviderError(err.to_string()))?
            .json()
            .await
            .map_err(|err| LLMWorkerErrors::ProviderError(err.to_string()))?;

        // Extract the generated text here
        Ok(response.result.response)
    }
}

impl LLMWorker for CloudflareWorker {
    fn generate_tier1(self, _content: String) -> String {
        "".to_string()
    }
    fn generate_tier2(self, _content: String) -> String {
        "".to_string()
    }
    fn generate_tier3(self, _content: String) -> String {
        "".to_string()
    }
}
