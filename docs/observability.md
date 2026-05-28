# Observability

This document describes the observability layer in the logag codebase.

## Architecture

Observability is managed through a singleton (`Observability`) exposed via `logag::observability`. It provides a global tracing subscriber that any module can use to emit structured logs, spans, and traces.

### Singleton Design

The `Observability::init()` function uses `std::sync::OnceLock` to ensure the tracing subscriber is initialized exactly once, even when called concurrently from multiple threads or tasks. Subsequent calls are no-ops and return the same `&'static Self`.

### Tracing Subscriber

The subscriber is composed of two stacked layers:

1. **`fmt::layer()`** — Formats and writes structured log output to stderr with timestamps and metadata.
2. **`EnvFilter`** — Reads the `RUST_LOG` environment variable to control log level filtering. Falls back to `debug` level if unset.

### Usage

```rust
// In binary entrypoint (called once):
use logag::observability::Observability;
Observability::init();

// Anywhere in the codebase (logs, spans, traces):
tracing::info!("something happened");
tracing::debug!(key = %value, "processed item");
tracing::warn!("deprecated path: {}", path);
let span = tracing::info_span!("request", id = %req_id);
```

### Environment Variables

| Variable    | Default | Description                          |
|-------------|---------|--------------------------------------|
| `RUST_LOG`  | `debug` | Log level filter (e.g. `info`, `warn`, `logag=trace`) |

### Modules

- **`src/observability.rs`** — Singleton definition and subscriber initialization.
- **`src/lib.rs`** — Re-exports the `observability` module as public.

### Thread Safety

The `OnceLock` primitive guarantees safe concurrent access. The `tracing` crate's subscriber is a global resource and is itself thread-safe, allowing multiple tasks to emit events simultaneously without coordination.
