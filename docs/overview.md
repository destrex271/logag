Based on the code changes from the past 24 hours, several new components have been introduced (such as the `RetrievalEngine` and new shared log modules), and existing components (like `EventAggregator` and `PostgresStorage`) have been refactored. 

Here are the documentation updates for the affected files in the `docs` folder.

---

### New File: `docs/retrieval_engine.md`

```markdown
# Retrieval Engine

The `RetrievalEngine` is a component responsible for querying the shared log to find and retrieve cached agent responses that correspond to similar user queries. It exposes these capabilities as Model Context Protocol (MCP) tools.

## Overview

When a user submits a query, the `RetrievalEngine` uses vector embeddings to perform a similarity search against previously recorded user inputs. If a close match is found, it retrieves the corresponding cached agent response, enabling efficient response reuse and caching.

## Structs and Types

### `RetrievalEngine`
The main service struct that wraps the `AgentRecorder` shared log.

```rust
pub struct RetrievalEngine {
    shared_log: Arc<AgentRecorder>,
}
```

### `ReadQuery`
The input parameter structure for retrieving cached responses.

```rust
pub struct ReadQuery {
    pub content: String,
}
```

## MCP Tools

### `get_cached_agent_response`
Retrieves the latest agent output stored for a semantically similar user query.

* **Description**: "Get latest agent output that was stored for similar user query."
* **Parameters**: `ReadQuery` containing the query `content`.
* **Returns**: `String` (The cached agent response, or an error message if no match is found).

```rust
#[tool(description = "Get latest agent output that was stored for similar user query.")]
pub fn get_cached_agent_response(
    &self,
    Parameters(ReadQuery { content }): Parameters<ReadQuery>,
) -> String
```
```

---

### Update File: `docs/event_aggregator.md`

```markdown
# Event Aggregator

The `EventAggregator` is responsible for receiving and processing incoming events (such as user inputs and agent outputs) and appending them to the shared log.

## Refactoring Updates

* **Decoupling**: The `AgentRecorder` struct and its associated tests have been moved out of `event_aggregator.rs` and relocated to the `shared_log::agent_recorder` module.
* **Imports**: `EventAggregator` now imports `AgentRecorder` from `crate::shared_log::agent_recorder::AgentRecorder`.

## Struct Definition

```rust
pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
}
```
```

---

### New File: `docs/shared_log/agent_recorder.md`

```markdown
# Agent Recorder

The `AgentRecorder` is the concrete implementation of the `SharedLog` trait. It manages the persistence of standalone events, event pairs (user query + agent response), and handles embedding generation and similarity lookups.

## Overview

`AgentRecorder` coordinates between:
1. **Embedding Service**: Generates vector embeddings for user inputs using `FastEmbeddingService`.
2. **Storage Engine**: Persists events, embeddings, and relationships into the configured database backend.

## Implementation of `SharedLog`

### `append_event`
Appends a standalone event to the storage backend asynchronously.

### `append_event_pair`
Appends a user input event and an agent output event. It triggers asynchronous embedding generation for the user input and links the agent response to the user query in the cache.

### `find_similar_user_event`
Generates an embedding for the provided query content and searches the storage engine for the closest matching user input.

```rust
fn find_similar_user_event(
    &self,
    user_content: String,
) -> Result<SlimUserEmbeddingInput, SharedLogErrors>
```

### `fetch_ai_response_for_user_event`
Retrieves the cached agent response associated with a specific user input ID.

```rust
fn fetch_ai_response_for_user_event(
    &self,
    user_input_id: uuid::Uuid,
) -> Result<LogContent, SharedLogErrors>
```
```

---

### Update File: `docs/shared_log/traits.md`

```markdown
# Shared Log Traits

This document defines the core traits used by the shared logging and event recording system.

## `SharedLog` Trait

The `SharedLog` trait has been expanded to support retrieval operations based on semantic similarity.

```rust
pub trait SharedLog {
    fn append_event(&self, event: Box<dyn Event>) -> uuid::Uuid;
    fn append_event_pair(&self, user_input: Box<dyn Event>, agent_output: Box<dyn Event>);
    
    /// Finds a user event semantically similar to the provided input string.
    fn find_similar_user_event(
        &self,
        user_input: String,
    ) -> Result<SlimUserEmbeddingInput, SharedLogErrors>;
    
    /// Fetches the recorded AI response linked to a specific user event ID.
    fn fetch_ai_response_for_user_event(
        &self,
        user_input_id: uuid::Uuid,
    ) -> Result<LogContent, SharedLogErrors>;
}
```

## `EventReference` Trait

A new trait introduced to represent references to existing events.

```rust
pub trait EventReference: Send + Sync {
    fn get_source_id(&self) -> uuid::Uuid;
}
```
```

---

### New File: `docs/shared_log/errors.md`

```markdown
# Shared Log Errors

Defines the error types encountered during shared log operations, embedding generation, and retrieval.

## `SharedLogErrors`

```rust
#[derive(Debug)]
pub enum SharedLogErrors {
    UnexpectedStorageLevelError(StorageEngineErrors),
    NoMatchingEntry(String),
    UnableToGenerateEmbeddings(String),
    UnknownError,
}
```

### Variants

* **`UnexpectedStorageLevelError`**: Wraps errors bubbled up from the underlying database/storage engine.
* **`NoMatchingEntry`**: Returned when a requested event or cached response does not exist.
* **`UnableToGenerateEmbeddings`**: Returned when the embedding service fails to vectorize the input text.
* **`UnknownError`**: Fallback for unspecified failures.
```

---

### New File: `docs/shared_log/user_embedding_model.md`

```markdown
# User Embedding Models

Data structures representing embeddings and cached relationships within the shared log.

## Structs

### `UserEmbeddingInput`
Represents the full embedding record stored in the database.

```rust
pub struct UserEmbeddingInput {
    id: uuid::Uuid,
    user_event_id: uuid::Uuid,
    embedding: Vec<f32>,
}
```

### `SlimUserEmbeddingInput`
A lightweight representation of a user embedding record, containing only the identifiers. Used for fast lookups and passing references.

```rust
pub struct SlimUserEmbeddingInput {
    pub id: uuid::Uuid,
    pub user_event_id: uuid::Uuid,
}
```

### `CachedAgentResponse`
Represents the mapping between a user query and the corresponding agent response.

```rust
pub struct CachedAgentResponse {
    id: uuid::Uuid,
    log_event_id: uuid::Uuid,
    user_query_id: uuid::Uuid,
}
```
```

---

### Update File: `docs/storage/traits.md`

```markdown
# Storage Engine Traits

The `StorageEngine` trait defines the interface that any database backend must implement.

## Updated `StorageEngine` Trait

The trait has been updated with two new asynchronous methods to support semantic search and response retrieval.

```rust
pub trait StorageEngine: Send + Sync + 'static {
    // ... existing methods ...

    /// Performs a vector similarity search to find the closest matching user input embedding.
    async fn get_similar_user_input_embedding(
        &self,
        embedding: Vec<f32>,
    ) -> Result<SlimUserEmbeddingInput, StorageEngineErrors>;

    /// Retrieves the cached agent response content associated with a user input ID.
    async fn get_agent_output_for_user_input(
        &self,
        user_input_id: uuid::Uuid,
    ) -> Result<LogContent, StorageEngineErrors>;
}
```
```

---

### Update File: `docs/storage/postgres.md`

```markdown
# Postgres Storage Backend

PostgreSQL implementation of the `StorageEngine` trait.

## Vector Search & Cache Retrieval

The Postgres backend utilizes the `pgvector` extension (via the `<=>` cosine distance operator) to perform similarity searches on user input embeddings.

### `get_similar_user_input_embedding`
Executes a cosine distance query against the `UserInputEmbedding` table to find the single closest match to the provided embedding vector.

```sql
WITH closest_matches AS (
    SELECT id, user_event_id 
    FROM UserInputEmbedding 
    ORDER BY embedding <=> $1::vector ASC 
    LIMIT 1
)
SELECT * FROM closest_matches ORDER BY id DESC;
```

### `get_agent_output_for_user_input`
Performs a two-step lookup:
1. Finds the `log_event_id` from the `CachedAgentResponse` table matching the given `user_query_id`.
2. Retrieves the text `content` from the `LogEvent` table matching that `log_event_id`.
```
