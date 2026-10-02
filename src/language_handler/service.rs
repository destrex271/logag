use crate::shared_log::agent_recorder::{self, AgentRecorder};
use regex::Regex;

const TOKEN_COUNT_THRESHOLD: isize = 50;

pub struct LanguageService {
    word_split_regex: Regex,
}

impl LanguageService {
    pub fn new() -> Self {
        LanguageService {
            word_split_regex: Regex::new(r"\W+").unwrap(),
        }
    }

    pub fn format_content(&self, content: &str) -> String {
        // TODO(destrex271): Use NLP techniques like stemming etc to improve data quality.
        let new_content = content.trim().to_lowercase();
        new_content.to_string()
    }

    pub fn is_input_valueable(&self, content: &str) -> bool {
        // This is naive and NOT AT ALL A GOOD CHECK.
        if content.len() == 0 || self.split_tokens(content).len() as isize > TOKEN_COUNT_THRESHOLD {
            return false;
        }
        true
    }

    fn split_tokens(&self, content: &str) -> Vec<String> {
        return self
            .word_split_regex
            .split(content)
            .filter(|s| !s.is_empty())
            .collect();
    }
}

impl Default for LanguageService {
    fn default() -> Self {
        LanguageService::new()
    }
}

impl Clone for LanguageService {
    fn clone(&self) -> Self {
        LanguageService::new()
    }
}
