use crate::embeddings::traits::EmbeddingsService;
use fastembed::TextEmbedding;

pub struct FastEmbeddingService {
    model: TextEmbedding,
}

impl EmbeddingsService for FastEmbeddingService {
    fn generate_embeddings(&mut self, documents: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        match self.model.embed(documents, None) {
            Ok(content) => {
                tracing::info!("Successfully generated embeddings {}", content.len());
                Ok(content)
            }
            Err(err) => {
                let msg = format!("unable to initialize model: {:?}", err);
                tracing::error!(msg);
                Err(msg)
            }
        }
    }
}

impl FastEmbeddingService {
    pub fn new() -> Result<FastEmbeddingService, String> {
        match TextEmbedding::try_new(Default::default()) {
            Ok(model) => Ok(FastEmbeddingService { model }),
            Err(err) => {
                let msg = format!("unable to initialize model: {:?}", err);
                tracing::error!(msg);
                Err(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_new_success() {
        let service = FastEmbeddingService::new();
        assert!(service.is_ok());
    }

    #[test]
    #[serial]
    fn test_generate_embeddings_single() {
        let mut service = FastEmbeddingService::new().unwrap();
        let result = service.generate_embeddings(vec!["Hello world".to_string()]);
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 1);
        assert!(!embeddings[0].is_empty());
    }

    #[test]
    #[serial]
    fn test_generate_embeddings_empty_input() {
        let mut service = FastEmbeddingService::new().unwrap();
        let result = service.generate_embeddings(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    #[serial]
    fn test_generate_embeddings_multiple_documents() {
        let mut service = FastEmbeddingService::new().unwrap();
        let result = service.generate_embeddings(vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ]);
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        for (i, emb) in embeddings.iter().enumerate() {
            assert!(!emb.is_empty(), "Embedding {} is empty", i);
        }
    }

    #[test]
    #[serial]
    fn test_embeddings_have_consistent_dimensions() {
        let mut service = FastEmbeddingService::new().unwrap();
        let result = service.generate_embeddings(vec!["First".to_string(), "Second".to_string()]);
        assert!(result.is_ok());
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        let dim = embeddings[0].len();
        assert!(dim > 0);
        assert_eq!(embeddings[1].len(), dim);
    }
}
