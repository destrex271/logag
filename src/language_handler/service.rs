
pub struct LanguageService{}

impl LanguageService{
    pub fn new() -> Self{
        LanguageService {  }
    }
    pub fn format_content(&self, content: &str) -> String {
        // TODO(destrex271): Use NLP techniques like stemming etc to improve data quality.
        let new_content = content.trim().to_lowercase();
        new_content.to_string()
    }
}

impl Clone for LanguageService{
   fn clone(&self) -> Self {
       LanguageService::new() 
   } 
}
