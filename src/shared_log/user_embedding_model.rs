use uuid;

// pub struct UserEmbeddingInput {
//     id: uuid::Uuid,
//     user_event_id: uuid::Uuid,
//     embedding: Vec<f32>,
// }

pub struct SlimUserEmbeddingInput {
    pub user_event_id: uuid::Uuid,
}

// pub struct CachedAgentResponse {
//     id: uuid::Uuid,
//     log_event_id: uuid::Uuid,
//     user_query_id: uuid::Uuid,
// }
