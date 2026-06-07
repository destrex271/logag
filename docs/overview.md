Based on the code changes from the past 24 hours, here are the updates to the documentation. 

We have:
1. Created a new documentation file for the newly introduced `observability` module: `docs/observability.md`.
2. Updated the existing `docs/event_aggregator.md` to reflect the transition from standard standard output printing (`println!`) to structured logging (`tracing`).

---

### New File: `docs/observability.md`

```markdown
# Observability

The `observability` module provides a centralized, thread-safe mechanism for initializing application-wide logging and tracing. It configures structured diagnostics to help monitor and debug the system.

## Overview

The module exposes the `Observability` utility, which configures the `tracing` ecosystem. It ensures that logging subscribers are registered exactly once during the application lifecycle.

## Components

### `Observability`

A unit struct used to manage the initialization of the tracing subscriber.

#### Methods

##### `init() -> &'static Self`
Initializes the global tracing subscriber. 
* **Thread Safety**: Uses `std::sync::OnceLock` internally to guarantee that initialization logic is executed only once, even if called from multiple threads.
* **Configuration**:
  * Registers a formatting layer (`tracing_subscriber::fmt::layer()`) to format log events.
  * Registers an environment filter (`tracing_subscriber::EnvFilter`) which reads the log level from the environment (e.g., `RUST_LOG`). If no environment variable is set, it defaults to the `debug` log level.

## Usage Example

Initialize observability at the very beginning of the application entry point (typically in `main.rs`):

```rust
use logag::observability::Observability;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    Observability::init();

    // Application logic...
    Ok(())
}
```
```

---

### Updated File: `docs/event_aggregator.md`

```markdown
# Event Aggregator

The `event_aggregator` module is responsible for receiving, processing, and persisting system events. It coordinates event ingestion through the `EventAggregator` and handles event persistence via the `AgentRecorder`.

## Architecture

- **`EventAggregator`**: The entry point for adding events. It processes incoming events and routes them to the appropriate storage or recorder.
- **`AgentRecorder`**: Implements the `SharedLog` trait to append events to the underlying storage.

---

## Logging and Diagnostics

The module utilizes structured logging via the `tracing` crate to provide rich, queryable diagnostic information. 

### Key Logged Events

#### 1. Event Appended (`AgentRecorder::append_event`)
When an event is successfully appended to the log, an `INFO` level log is emitted with structured context:
* **Fields**:
  * `content`: The string representation of the event's content.
  * `event_type`: The type classification of the event.
  * `id`: The unique identifier of the event.
* **Message**: `"appended event"`

#### 2. Event Added (`EventAggregator::add_event`)
When a new event is added to the aggregator, an `INFO` level log is emitted:
* **Fields**:
  * `agent_notes`: Notes associated with the agent processing the event.
* **Message**: `"add_event called"`

*Note: Standard `println!` debugging statements have been replaced with these structured `tracing::info!` macros to support production-grade observability.*
```
