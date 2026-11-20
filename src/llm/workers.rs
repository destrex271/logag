pub trait LLMWorker {
    fn generate_tier1(self, content: String) -> String;
    fn generate_tier2(self, content: String) -> String;
    fn generate_tier3(self, content: String) -> String;
    // Tier 4 is the raw logag storage.
}

pub enum LLMWorkerErrors {
    ProviderError(String),
}
