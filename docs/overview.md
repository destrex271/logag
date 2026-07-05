Based on the code changes introduced in the past 24 hours, here are the new and updated documentation files within the `docs` folder.

---

### `docs/event_aggregator.md` (Updated)

```markdown
# Event Aggregator

The `EventAggregator` is responsible for receiving events from external clients, formatting the content, and appending them to the shared log.

## Responsibilities

- **Event Ingestion**: Exposes endpoints/tools to record standalone events or event pairs (user input and agent output).
- **Content Normalization**: Integrates with `LanguageService` to format and clean incoming event content before storage.
- **Log Delegation**: Forwards processed events to the `AgentRecorder` (which implements `SharedLog`) for persistence and embedding generation.

## Key Structs

### `EventAggregator`

```rust
pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
    lang_service: LanguageService,
}
```

## Core Methods

### `add_event`
Ingests a single event, formats its content using the `LanguageService`, and appends it to the shared log.
- **Parameters**: `Parameters<MCPEvent>`
- **Returns**: `String` (Event ID)

### `add_event_pair`
Ingests a pair of events representing a user query and the corresponding agent response. Both contents are formatted before being appended as a pair.
- **Parameters**: `Parameters<MCPEventPair>`
- **Returns**: `String` (User Event ID)
```

---

### `docs/language_handler/service.md` (New File)

```markdown
# Language Service

The `LanguageService` provides text preprocessing and natural language processing (NLP) utilities to normalize event content before it is stored or embedded.

## Key Structs

### `LanguageService`

```rust
pub struct LanguageService {}
```

## Core Methods

### `format_content`
Normalizes input text to ensure consistency across storage and vector search.
- **Parameters**: `content: &str`
- **Returns**: `String`
- **Current Implementation**: Trims leading/trailing whitespace and converts the text to lowercase.
- **Future Enhancements**: Planned integration of NLP techniques such as stemming and lemmatization to improve data quality.
```

---

### `docs/retrieval_engine.md` (New File)

```markdown
# Retrieval Engine

The `RetrievalEngine` is a query-facing component that allows clients to search and retrieve cached agent responses based on semantic similarity to a user query.

## Endpoint Integration
The retrieval engine is exposed as a dedicated Model Context Protocol (MCP) service at the `/read_mcp` path.

## Key Structs

### `RetrievalEngine`

```rust
#[derive(Clone)]
pub struct RetrievalEngine {
    shared_log: Arc<AgentRecorder>,
    lang_service: LanguageService,
}
```

### `ReadQuery`

```rust
#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub struct ReadQuery {
    content: String
}
```

## Tools

### `get_cached_agent_response`
Finds the most semantically similar historical user query and returns the cached agent response associated with it.
- **Description**: "Get latest agent output that was stored for similar user query."
- **Workflow**:
  1. Formats the incoming query content using `LanguageService`.
  2. Queries the `AgentRecorder` to find a similar user event.
  3. Fetches the corresponding AI response from the storage backend.
- **Returns**: `String` (The cached agent response content or an error message).
```

---

### `docs/shared_log/overview.md` (Updated)

```markdown
# Shared Log Overview

The `shared_log` module manages the lifecycle of event logging, embedding generation, and semantic retrieval. It acts as the intermediary between high-level services (like `EventAggregator` and `RetrievalEngine`) and the underlying storage engines.

## Components

- **[Agent Recorder](agent_recorder.md)**: The core implementation of the `SharedLog` trait, handling asynchronous storage and embedding generation.
- **[Errors](errors.md)**: Centralized error definitions for shared log operations.
- **[User Embedding Model](user_embedding_model.md)**: Data structures representing vector embeddings and cached relationships.
- **[Log](log.md)**: Concrete implementations of events and log content.
- **[Traits](traits.md)**: Shared interfaces for logs and events.
```

---

### `docs/shared_log/agent_recorder.md` (New File)

```markdown
# Agent Recorder

The `AgentRecorder` implements the `SharedLog` trait. It coordinates storing raw events, generating vector embeddings for user queries, and linking user queries to agent responses.

## Key Structs

### `AgentRecorder`

```rust
pub struct AgentRecorder {
    storage: std::sync::Arc<OnceCell<Box<dyn StorageEngine>>>,
    embedding_model: std::sync::Arc<OnceCell<std::sync::Mutex<Box<dyn EmbeddingsService>>>>,
}
```

## Trait Implementation: `SharedLog`

### `append_event`
Appends a standalone event to the log and asynchronously persists it to the storage engine.

### `append_event_pair`
Appends a user input event and an agent output event. It triggers asynchronous vector embedding generation for the user input and stores the relationship in the cache.

### `find_similar_user_event`
Generates an embedding for the provided query and searches the storage engine for the closest matching historical user input.
- **Parameters**: `user_content: String`
- **Returns**: `Result<SlimUserEmbeddingInput, SharedLogErrors>`

### `fetch_ai_response_for_user_event`
Retrieves the cached agent response associated with a specific user event ID.
- **Parameters**: `user_input_id: uuid::Uuid`
- **Returns**: `Result<LogContent, SharedLogErrors>`
```

---

### `docs/shared_log/errors.md` (New File)

```markdown
# Shared Log Errors

Defines the error types encountered during shared log operations, embedding generation, and retrieval.

## `SharedLogErrors`

```rust
pub enum SharedLogErrors {
    UnexpectedStorageLevelError(StorageEngineErrors),
    NoMatchingEntry(String),
    UnableToGenerateEmbeddings(String),
    UnknownError,
}
```
```

---

### `docs/shared_log/user_embedding_model.md` (New File)

```markdown
# User Embedding Model

Data structures representing vector embeddings and cached agent responses in the system.

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
A lightweight representation of an embedding record, excluding the heavy vector data, used for quick lookups and references.
```rust
pub struct SlimUserEmbeddingInput {
    pub id: uuid::Uuid,
    pub user_event_id: uuid::Uuid,
}
```

### `CachedAgentResponse`
Represents the mapping between a user query and the agent's response.
```rust
pub struct CachedAgentResponse {
    id: uuid::Uuid,
    log_event_id: uuid::Uuid,
    user_query_id: uuid::Uuid,
}
```
```

---

### `docs/storage/traits.md` (Updated)

```markdown
# Storage Traits

The `StorageEngine` trait defines the interface that any storage backend (e.g., Postgres) must implement.

## Updated Trait Methods

The following methods have been added to support semantic search and response caching:

### `get_similar_user_input_embedding`
Performs a vector similarity search against stored user input embeddings.
```rust
async fn get_similar_user_input_embedding(
    &self,
    embedding: Vec<f32>,
) -> Result<SlimUserEmbeddingInput, StorageEngineErrors>;
```

### `get_agent_output_for_user_input`
Retrieves the cached agent response content associated with a given user input event ID.
```rust
async fn get_agent_output_for_user_input(
    &self,
    user_input_id: uuid::Uuid,
) -> Result<LogContent, StorageEngineErrors>;
```
```

---

### `docs/storage/postgres.md` (Updated)

```markdown
# Postgres Storage Backend

The `PostgresStorage` engine implements the `StorageEngine` trait, utilizing PostgreSQL with the `pgvector` extension for vector similarity searches.

## New Implementations

### Vector Similarity Search (`get_similar_user_input_embedding`)
Uses the `pgvector` cosine distance operator (`<=>`) to find the closest matching user input embedding:
```sql
WITH closest_matches AS (
    SELECT id, user_event_id 
    FROM UserInputEmbedding 
    ORDER BY embedding <=> $1::vector ASC 
    LIMIT 1
)
SELECT * FROM closest_matches ORDER BY id DESC;
```

### Cached Response Retrieval (`get_agent_output_for_user_input`)
Performs a two-step lookup:
1. Finds the `log_event_id` from `CachedAgentResponse` where `user_query_id` matches.
2. Fetches the raw text content from the `LogEvent` table using that `log_event_id`.
```
