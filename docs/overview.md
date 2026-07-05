Based on the code changes from the past 24 hours, several new components have been introduced (such as the `RetrievalEngine` and `LanguageService`), and existing components (like `AgentRecorder` and `PostgresStorage`) have been refactored or expanded to support vector similarity search and cached response retrieval.

Here are the new and updated documentation files reflecting these changes.

---

### New File: `docs/language_handler/service.md`

```markdown
# Language Handler Service

The `LanguageService` is responsible for preprocessing and normalizing text content before it is stored, indexed, or used for embedding generation. This ensures consistency in data quality and improves the accuracy of downstream NLP and vector search operations.

## Struct Definition

```rust
pub struct LanguageService {}
```

## Key Methods

### `new`
Initializes a new instance of the `LanguageService`.
```rust
pub fn new() -> Self
```

### `format_content`
Normalizes the input string by trimming leading/trailing whitespace and converting all characters to lowercase. 
```rust
pub fn format_content(&self, content: &str) -> String
```
* **Future Enhancements**: This method is designed to eventually incorporate advanced NLP techniques such as stemming, lemmatization, and stop-word removal to further improve vector search quality.

## Traits Implemented

* `Clone`: Allows the service to be easily shared across multi-threaded contexts (e.g., within the `EventAggregator` and `RetrievalEngine`).
```

---

### New File: `docs/retrieval_engine.md`

```markdown
# Retrieval Engine

The `RetrievalEngine` is a core component that handles semantic search and cached response retrieval. It exposes Model Context Protocol (MCP) tools to allow clients to query previously recorded agent interactions based on semantic similarity.

## Struct Definition

```rust
#[derive(Clone)]
pub struct RetrievalEngine {
    shared_log: Arc<AgentRecorder>,
    lang_service: LanguageService,
}
```

## Initialization

The `RetrievalEngine` is initialized with a `GlobalConfig` and sets up its own connection to the `AgentRecorder` (which manages storage and embedding generation).

```rust
pub fn new(global_config: GlobalConfig) -> Self
```

## MCP Tools

### `get_cached_agent_response`
Exposed as an MCP tool to retrieve the closest matching cached agent response for a given user query.

* **Description**: "Get latest agent output that was stored for similar user query."
* **Parameters**: `ReadQuery` containing the raw query `content`.
* **Workflow**:
  1. Normalizes the input query using `LanguageService`.
  2. Generates vector embeddings for the query.
  3. Queries the storage backend for the nearest neighbor embedding in the `UserInputEmbedding` table.
  4. Fetches the corresponding agent response from the `CachedAgentResponse` table.
  5. Returns the cached text response, or an error message if no match is found.

```rust
#[tool(description = "Get latest agent output that was stored for similar user query.")]
pub fn get_cached_agent_response(
    &self,
    Parameters(ReadQuery { content }): Parameters<ReadQuery>,
) -> String
```
```

---

### New File: `docs/shared_log/agent_recorder.md`

```markdown
# Agent Recorder

The `AgentRecorder` implements the `SharedLog` trait and acts as the orchestrator for appending events, generating vector embeddings, and querying the storage engine for semantic matches.

## Struct Definition

```rust
pub struct AgentRecorder {
    storage: std::sync::Arc<OnceCell<Box<dyn StorageEngine>>>,
    embedding_model: std::sync::Arc<OnceCell<std::sync::Mutex<Box<dyn EmbeddingsService>>>>,
}
```

## Key Methods & Trait Implementations

### `SharedLog` Implementation

#### `append_event`
Appends a single standalone event to the log and asynchronously persists it to the configured storage engine.
```rust
fn append_event(&self, event: Box<dyn Event>) -> uuid::Uuid
```

#### `append_event_pair`
Appends a user input event and its corresponding agent output event. It automatically triggers asynchronous embedding generation for the user input and stores the relationship in the cache tables.
```rust
fn append_event_pair(&self, user_input: Box<dyn Event>, agent_output: Box<dyn Event>)
```

#### `find_similar_user_event`
Generates vector embeddings for the provided query text and searches the storage engine for the most semantically similar user event.
```rust
fn find_similar_user_event(
    &self,
    user_content: String,
) -> Result<SlimUserEmbeddingInput, SharedLogErrors>
```

#### `fetch_ai_response_for_user_event`
Retrieves the cached agent response associated with a specific user event ID.
```rust
fn fetch_ai_response_for_user_event(
    &self,
    user_input_id: uuid::Uuid,
) -> Result<LogContent, SharedLogErrors>
```
```

---

### Updated File: `docs/event_aggregator.md`

```markdown
# Event Aggregator

The `EventAggregator` is responsible for receiving incoming events via MCP, normalizing their content, and routing them to the shared log recorder.

## Struct Definition

```rust
pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: std::sync::Arc<AgentRecorder>,
    lang_service: LanguageService,
}
```

## Key Changes

* **Decoupling of AgentRecorder**: The `AgentRecorder` implementation has been moved to its own module (`crate::shared_log::agent_recorder`) to separate concerns.
* **Content Normalization**: All incoming event contents are now preprocessed using `LanguageService::format_content` before being written to the log. This ensures that stored logs and generated embeddings are consistently formatted (trimmed and lowercased).

## Key Methods

### `add_event`
Formats the incoming content and appends a single event to the log.
```rust
pub fn add_event(&self, Parameters(MCPEvent { ... }): Parameters<MCPEvent>) -> String
```

### `add_event_pair`
Formats both the user and agent contents and appends them as a linked pair.
```rust
pub fn add_event_pair(&self, Parameters(MCPEventPair { ... }): Parameters<MCPEventPair>) -> String
```
```

---

### Updated File: `docs/storage/traits.md`

```markdown
# Storage Engine Traits

The `StorageEngine` trait defines the interface that any storage backend (e.g., Postgres, In-Memory) must implement.

## New Trait Methods

To support semantic caching and retrieval, the following asynchronous methods have been added to the `StorageEngine` trait:

### `get_similar_user_input_embedding`
Queries the database for the closest vector match to the provided embedding.
```rust
async fn get_similar_user_input_embedding(
    &self,
    embedding: Vec<f32>,
) -> Result<SlimUserEmbeddingInput, StorageEngineErrors>;
```

### `get_agent_output_for_user_input`
Retrieves the cached agent response text associated with a given user input event ID.
```rust
async fn get_agent_output_for_user_input(
    &self,
    user_input_id: uuid::Uuid,
) -> Result<LogContent, StorageEngineErrors>;
```
```

---

### Updated File: `docs/storage/postgres.md`

```markdown
# Postgres Storage Backend

The `PostgresStorage` backend implements the `StorageEngine` trait, utilizing `sqlx` and `pgvector` to store and query high-dimensional vector embeddings.

## Vector Similarity Search Implementation

### `get_similar_user_input_embedding`
Uses the `pgvector` cosine distance operator (`<=>`) to perform a nearest-neighbor search on the `UserInputEmbedding` table.

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
1. Finds the `log_event_id` (agent output ID) associated with the `user_query_id` in the `CachedAgentResponse` table.
2. Fetches the raw text `content` from the `LogEvent` table matching that `log_event_id`.
```
