Based on the code changes introduced in the past 24 hours, here are the new and updated documentation files reflecting the architecture, configuration, and storage backend updates.

---

### New File: `docs/global_config.md`

```markdown
# Global Configuration

The `global_config` module defines the configuration structure and loading mechanism for the application. Configuration is typically defined in a TOML file and loaded at startup.

## GlobalConfig Struct

Represents the global configuration settings for the application.

### Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `storage_backend` | `StorageBackend` | The storage engine backend to use (e.g., `Postgres`). |
| `database_connection_string` | `String` | The connection string used to connect to the database. |

### Methods

#### `load_config`
```rust
pub fn load_config(file_path: String) -> GlobalConfig
```
Loads and parses a TOML configuration file from the specified file path. Panics if the file cannot be opened, read, or parsed.

## Usage Example

```rust
let config = GlobalConfig::load_config("config/default.toml".to_string());
```
```

---

### Updated File: `docs/event_aggregator.md`

```markdown
# Event Aggregator

The `EventAggregator` is the central component responsible for receiving, processing, and routing events. It implements the tool routing interface and delegates event persistence to an asynchronous recorder.

## EventAggregator Struct

```rust
pub struct EventAggregator {
    event_factory: EventFactory,
    global_config: GlobalConfig,
    shared_log: AgentRecorder,
}
```

### Methods

#### `new`
```rust
pub fn new(global_config: GlobalConfig) -> Self
```
Initializes a new `EventAggregator` instance, creating and initializing an internal `AgentRecorder` with the provided configuration.

#### `log_event`
```rust
pub async fn log_event(
    &self,
    event_type: EventType,
    content: String,
    unix_epoch_timestamp: String,
    agent_notes: String,
) -> String
```
An MCP tool handler that creates a log event and asynchronously appends it to the shared log.

---

## Internal Components

### AgentRecorder

An internal helper struct that manages the storage engine connection and handles asynchronous event persistence.

```rust
struct AgentRecorder {
    storage: std::sync::Arc<Mutex<Option<Box<dyn StorageEngine>>>>,
}
```

*   **Asynchronous Initialization**: When initialized, it spawns a background Tokio task to load the storage backend without blocking the main thread.
*   **Asynchronous Appending**: `append_event` spawns a background task to write the event to the active storage engine.

---

## MCPEvent Struct

The data transfer object received by the `log_event` tool.

| Field | Type | Description |
| :--- | :--- | :--- |
| `event_type` | `EventType` | Type of the event (`UserInput` or `AgentOutput`). |
| `content` | `String` | The main content of the log. |
| `unix_epoch_timestamp` | `String` | Unix epoch timestamp representing when the event occurred. |
| `agent_notes` | `String` | Additional metadata or notes from the agent. |
```

---

### Updated File: `docs/shared_log/traits.md`

```markdown
# Shared Log Traits

Defines the core abstractions for events and log recorders within the system.

## EventType Enum

Represents the source/type of the logged event.

*   `UserInput` (serialized/displayed as `"user_input"`)
*   `AgentOutput` (serialized/displayed as `"agent_output"`)

## SharedLog Trait

```rust
pub trait SharedLog {
    fn append_event(&self, event: Box<dyn Event>);
}
```

## Event Trait

Represents a log event. Must be thread-safe (`Send + Sync`).

```rust
#[async_trait::async_trait]
pub trait Event: Send + Sync {
    fn get_event_type(&self) -> EventType;
    fn get_content(&self) -> String;
    fn get_id(&self) -> uuid::Uuid; // Returns a UUID v7
    fn get_timestamp(&self) -> isize;
}
```

## EventFactory Struct

A factory utility to simplify the creation of `Event` trait objects.

### Methods

#### `create_log_event`
```rust
pub fn create_log_event(
    &self,
    content: String,
    timestamp: String,
    event_type: EventType,
) -> Box<dyn Event>
```
```

---

### Updated File: `docs/shared_log/log.md`

```markdown
# Log Event Implementation

This module provides the concrete implementation of the `Event` trait.

## LogEvent Struct

```rust
pub struct LogEvent {
    event_type: EventType,
    content: LogContent,
    timestamp: String,
}
```

### Dynamic ID Generation
Unlike previous versions where IDs were static strings stored in the struct, `LogEvent` now dynamically generates a **UUID v7** based on its timestamp when `get_id()` is called:

```rust
fn get_id(&self) -> uuid::Uuid {
    let seconds = self.get_timestamp();
    uuid::Uuid::new_v7(Timestamp::from_unix_time(seconds as u64, 0, 0, 0))
}
```

### Methods

#### `new`
```rust
pub fn new(event_type: EventType, content: String, timestamp: String) -> Self
```
Creates a new `LogEvent` instance.
```

---

### Updated File: `docs/storage/overview.md`

```markdown
# Storage Module Overview

The `storage` module provides the persistence layer for the application's event log. 

## Supported Backends

*   **Postgres**: A robust, production-ready relational database backend utilizing `sqlx` with connection pooling and automatic migrations.
*   *(Note: The experimental DuckDB backend has been removed).*

## Architecture

The storage layer is abstracted behind the `StorageEngine` trait. At startup, the application uses the `StorageBackendProvider` to resolve and instantiate the configured storage engine based on the `GlobalConfig`.
```

---

### New File: `docs/storage/postgres.md`

```markdown
# Postgres Storage Engine

The `PostgresStorage` struct implements the `StorageEngine` trait, providing persistent storage in a PostgreSQL database.

## PostgresStorage Struct

```rust
pub struct PostgresStorage {
    connection_string: String,
    connection_pool: Pool<Postgres>,
}
```

### Connection Management
*   **Pool Size**: Configured with a maximum of 5 connections.
*   **Lazy Connection**: Connections are established lazily to prevent blocking startup.
*   **Migrations**: Automatically runs database migrations located in `./migrations` upon initialization.

## StorageEngine Implementation

### `load_storage`
```rust
async fn load_storage(config: GlobalConfig) -> Self
```
Initializes the connection pool and runs pending database migrations.

### `store_event`
```rust
async fn store_event(&self, event: &dyn Event) -> Result<(), StorageEngineErrors>
```
Inserts a log event into the `LogEvent` table. Converts the event's integer timestamp into a UTC DateTime before insertion.

### `get_events`
```rust
async fn get_events<T, F>(
    &self,
    from_timestamp: isize,
    to_timestamp: isize,
    factory_fn: F,
) -> Result<Vec<T>, StorageEngineErrors>
where
    T: Event,
    F: Fn(uuid::Uuid, String, isize, String) -> T + Send
```
Retrieves events within a specified timestamp range and reconstructs them using the provided factory function.
```

---

### Updated File: `docs/storage/traits.md`

```markdown
# Storage Traits and Types

This module defines the interfaces, error types, and providers for the storage layer.

## StorageBackend Enum

Supported database backends.

```rust
pub enum StorageBackend {
    Postgres,
}
```

## StorageBackendProvider

Resolves and constructs the concrete storage engine based on the global configuration.

```rust
impl StorageBackendProvider {
    pub async fn get_storage_backend(config: GlobalConfig) -> Box<dyn StorageEngine>
}
```

## StorageEngineErrors Enum

Represents errors that can occur during storage operations.

```rust
pub enum StorageEngineErrors {
    InvalidTimestamp(isize),
    DatabaseError(Box<dyn Error>),
    NoDataForField(String),
    UnableToAcquireConnection(String),
    UnableToExecuteMigrations(String),
}
```

## StorageEngine Trait

The core interface for all storage backends.

```rust
#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync + 'static {
    async fn load_storage(config: GlobalConfig) -> Self where Self: Sized;
    async fn store_event(&self, event: &dyn Event) -> Result<(), StorageEngineErrors>;
    async fn get_events<T, F>(&self, from_timestamp: isize, to_timestamp: isize, factory_fn: F) -> Result<Vec<T>, StorageEngineErrors>
        where
            T: Event,
            F: Fn(uuid::Uuid, String, isize, String) -> T + Send,
            Self: Sized;
}
```
```
