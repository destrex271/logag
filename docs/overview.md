Based on the code changes from the past 24 hours, several major architectural updates have been introduced:
1. **Extraction of `AgentRecorder`**: Refactored out of `event_aggregator.rs` into its own module under `src/shared_log/agent_recorder.rs`.
2. **Introduction of `RetrievalEngine`**: A new component that exposes a read-only MCP service (`/read_mcp`) and a tool (`get_cached_agent_response`) to retrieve cached agent responses based on semantic similarity.
3. **Vector Similarity Search & Caching**: Added database-level support in `PostgresStorage` and the `StorageEngine` trait to query similar user embeddings and fetch corresponding cached agent outputs.
4. **New Shared Log Models and Errors**: Introduced `SharedLogErrors` and embedding-related data structures (`SlimUserEmbeddingInput`, etc.).

Below are the updated and newly created documentation files within the `docs/` folder structure.

---

### New File: `docs/retrieval_engine.md`

```markdown
# Retrieval Engine

The `RetrievalEngine` is a core component responsible for querying historical interaction logs and serving cached agent responses based on semantic similarity. It exposes a read-only Model Context Protocol (MCP) service.

## Overview

When a user submits a query, the `RetrievalEngine` uses vector embeddings to find the most semantically similar historical user query stored in the database. If a close match is found, it retrieves and returns the corresponding agent response that was previously generated and cached.

## Endpoint

The retrieval engine is exposed as an independent MCP service:
* **Route**: `/read_mcp`
* **Protocol**: Streamable HTTP Service (MCP)

## Structs and Types

### `RetrievalEngine`
The main engine struct holding a reference to the shared log recorder.
```rust
pub struct RetrievalEngine {
    shared_log: Arc<AgentRecorder>,
}
```

### `ReadQuery`
The input parameter structure for querying the cache.
```rust
pub struct ReadQuery {
    content: String,
}
```

## MCP Tools

### `get_cached_agent_response`
Retrieves the latest agent output stored for a semantically similar user query.

* **Description**: "Get latest agent output that was stored for similar user query."
* **Parameters**: `ReadQuery` (containing the raw query string `content`).
* **Returns**: `String` (The cached agent response, or an error message if no match is found).

### Workflow
1. Generates a vector embedding for the incoming `content` query.
2. Queries the storage backend for the closest matching user input embedding.
3. If a match is found, retrieves the associated cached agent response.
4. Returns the content of the agent response.
```

---

### New File: `docs/shared_log/agent_recorder.md`

```markdown
# Agent Recorder

The `AgentRecorder` implements the `SharedLog` trait. It is responsible for writing events to the storage engine, generating vector embeddings for user inputs, and querying the log for similar historical events.

> **Note**: This component was previously defined inside `event_aggregator.rs` and has been refactored into its own module under `src/shared_log/agent_recorder.rs`.

## Initialization

The `AgentRecorder` is initialized asynchronously with a `GlobalConfig` to set up the storage backend and the embedding service.

```rust
pub struct AgentRecorder {
    storage: Arc<OnceCell<Box<dyn StorageEngine>>>,
    embedding_model: Arc<OnceCell<Mutex<Box<dyn EmbeddingsService>>>>,
}
```

## Key Methods

### `append_event`
Appends a standalone event to the log and persists it to the configured storage engine.
* **Returns**: `uuid::Uuid` of the appended event.

### `append_event_pair`
Appends a pair of events (a `UserInput` and its corresponding `AgentOutput`).
1. Persists both events.
2. Generates vector embeddings for the `UserInput` content using `FastEmbeddingService`.
3. Asynchronously stores the user input embedding and links the agent response to the user query in the cache table.

### `find_similar_user_event`
Generates an embedding for the provided query string and searches the storage engine for the closest matching historical user input.
* **Returns**: `Result<SlimUserEmbeddingInput, SharedLogErrors>`

### `fetch_ai_response_for_user_event`
Retrieves the cached agent response associated with a specific user input event ID.
* **Returns**: `Result<LogContent, SharedLogErrors>`
```

---

### Updated File: `docs/event_aggregator.md`

```markdown
# Event Aggregator

The `EventAggregator` is the primary write-path component for capturing and logging events. It exposes an MCP service to record standalone events or event pairs (user inputs and agent outputs).

## Refactoring Updates
* **`AgentRecorder` Extraction**: The internal `AgentRecorder` struct and its associated storage/embedding logic have been moved to `src/shared_log/agent_recorder.rs`. `EventAggregator` now imports and utilizes `AgentRecorder` from the shared log module.

## Endpoint
* **Route**: `/mcp`
* **Protocol**: Streamable HTTP Service (MCP)

## Structs

### `EventAggregator`
```rust
pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
}
```
*(For details on how events are recorded and embedded, see the [Agent Recorder Documentation](./shared_log/agent_recorder.md).)*
```

---

### New File: `docs/shared_log/traits.md`

```markdown
# Shared Log Traits

Defines the core interfaces for logging and retrieving events within the system.

## `SharedLog` Trait

The `SharedLog` trait has been expanded to support read/retrieval operations alongside append operations.

```rust
pub trait SharedLog {
    fn append_event(&self, event: Box<dyn Event>) -> uuid::Uuid;
    
    fn append_event_pair(&self, user_input: Box<dyn Event>, agent_output: Box<dyn Event>);
    
    fn find_similar_user_event(
        &self,
        user_input: String,
    ) -> Result<SlimUserEmbeddingInput, SharedLogErrors>;
    
    fn fetch_ai_response_for_user_event(
        &self,
        user_input_id: uuid::Uuid,
    ) -> Result<LogContent, SharedLogErrors>;
}
```

## `EventReference` Trait

A new trait representing a reference to another event.

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

Defines the error types encountered during shared log operations, particularly during embedding generation and cache retrieval.

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
* **`UnexpectedStorageLevelError`**: Wraps underlying database/storage errors.
* **`NoMatchingEntry`**: Returned when no cached agent response exists for a given user event ID.
* **`UnableToGenerateEmbeddings`**: Returned if the embedding service fails to vectorize the input text.
```

---

### New File: `docs/shared_log/user_embedding_model.md`

```markdown
# User Embedding Models

Data structures representing database records for user input embeddings and cached agent responses.

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
A lightweight representation of an embedding record, omitting the raw vector data. Used for quick lookups and references.
```rust
pub struct SlimUserEmbeddingInput {
    pub id: uuid::Uuid,
    pub user_event_id: uuid::Uuid,
}
```

### `CachedAgentResponse`
Represents the mapping between a cached agent response and the user query that triggered it.
```rust
pub struct CachedAgentResponse {
    id: uuid::Uuid,
    log_event_id: uuid::Uuid,
    user_query_id: uuid::Uuid,
}
```
```

---

### Updated File: `docs/storage/traits.md`

```markdown
# Storage Engine Traits

The `StorageEngine` trait defines the interface that any storage backend (e.g., PostgreSQL) must implement.

## Updated Trait Methods

The following methods have been added to the `StorageEngine` trait to support semantic search and response retrieval:

```rust
#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync + 'static {
    // ... existing methods ...

    /// Performs a vector similarity search to find the closest matching user input embedding.
    async fn get_similar_user_input_embedding(
        &self,
        embedding: Vec<f32>,
    ) -> Result<SlimUserEmbeddingInput, StorageEngineErrors>;

    /// Retrieves the cached agent response content associated with a specific user input event ID.
    async fn get_agent_output_for_user_input(
        &self,
        user_input_id: uuid::Uuid,
    ) -> Result<LogContent, StorageEngineErrors>;
}
```
```

---

### Updated File: `docs/storage/postgres.md`

```markdown
# PostgreSQL Storage Backend

The `PostgresStorage` struct implements the `StorageEngine` trait, providing persistent storage using PostgreSQL and vector similarity search capabilities.

## Vector Search Implementation

`PostgresStorage` utilizes the pgvector extension (specifically the `<=>` cosine distance operator) to perform semantic similarity searches.

### `get_similar_user_input_embedding`
Executes a query to find the closest matching vector in the `UserInputEmbedding` table.
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
Retrieves the cached agent response content by:
1. Querying the `CachedAgentResponse` table for the `log_event_id` associated with the given `user_query_id`.
2. Querying the `LogEvent` table to fetch the raw text `content` of that agent event.
```
