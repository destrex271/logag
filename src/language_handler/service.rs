use crate::shared_log::agent_recorder::{self, AgentRecorder};
use regex::Regex;

const TOKEN_COUNT_THRESHOLD: isize = 30;

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
        let formatted_content = self.format_content(content);
        // This is naive and NOT AT ALL A GOOD CHECK.
        if formatted_content.len() == 0
            || self.split_tokens(&formatted_content).len() as isize > TOKEN_COUNT_THRESHOLD
        {
            return false;
        }
        true
    }

    fn split_tokens(&self, content: &str) -> Vec<String> {
        return self
            .word_split_regex
            .split(content)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
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

mod tests {
    #[test]
    fn test_is_input_valueable() {
        let service = LanguageService::new();
        assert!(!service.is_input_valueable(""));
        assert!(
            !service.is_input_valueable("a ".repeat(TOKEN_COUNT_THRESHOLD as usize + 1).as_str())
        );
        assert!(service.is_input_valueable("a ".repeat(TOKEN_COUNT_THRESHOLD as usize).as_str()));
    }

    #[test]
    fn test_format_content() {
        let service = LanguageService::new();
        assert_eq!(service.format_content("  Hello, World!  "), "hello, world!");
        assert_eq!(service.format_content(""), "");
    }
}
