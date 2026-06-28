
pub trait EmbeddingsService: Send + Sync + 'static{
    fn generate_embeddings(&mut self, documents: Vec<String>) -> Result<Vec<Vec<f32>>, String>;
}
