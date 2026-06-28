use crate::embeddings::traits::EmbeddingsService;
use fastembed::TextEmbedding;


pub struct FastEmbeddingService{
    model: TextEmbedding
}


impl EmbeddingsService for FastEmbeddingService{
    fn generate_embeddings(
        &mut self,
        documents: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, String> {
        match self.model.embed(documents, None){
            Ok(content) => {
                tracing::info!("Successfully generated embeddings {}", content.len());
                Ok(content)
            },
            Err(err) => {
                let msg = format!("unable to initialize model: {:?}", err);
                tracing::error!(msg);
                Err(msg)
            }
        }
    }
}

impl FastEmbeddingService{
    pub fn new() -> Result<FastEmbeddingService, String> {
        match TextEmbedding::try_new(Default::default()){
            Ok(model) => Ok( 
                FastEmbeddingService{
                    model: model,
                }
            ),
            Err(err) => {
                let msg = format!("unable to initialize model: {:?}", err);
                tracing::error!(msg);
                Err(msg)
            }
        }
    }
}
