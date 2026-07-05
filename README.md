# LogAg — Agentic Memory Optimizer

LogAg is an agentic memory optimization tool that records user–LLM interactions, builds a semantic cache of past queries and responses, and retrieves relevant context for new sessions — reducing token waste and improving agent continuity.

## Features

- [X] **Semantic caching** — stores user queries and agent responses, retrieves similar past interactions using vector embeddings (pgvector + fastembed)
- [ ] **Agent continuity** — if an agent crashes mid-run, a new spawn resumes as if it were a clone of its predecessor
- [X] **Dual protocol support** — serves via MCP (Model Context Protocol) and HTTP (Axum)
- [X] **Pluggable backends** — storage and embedding services are trait-based for easy swapping
- [ ]**Self-improving** — each loop leverages understanding developed in previous runs
- [ ] **Pluggable Storage Backends**: Currently supports only PostgreSQL. Support for Duckdb upcoming.

## Architecture

```mermaid
flowchart LR
    A[User / AI Client] --> B{MCP}
    A --> C{HTTP}
    B --> D[retrieval_engine]
    C --> E[api_handler]
    D --> F[AgentRecorder]
    E --> F
    F --> G[(Postgres + pgvector)]
    F --> H[fastembed]
    G --> I[LogEvent DB]
    I --> J[Context for next prompt]
```

## Installation

### Prerequisites

- Rust 2024 edition (`rustc` ≥ 1.85)
- PostgreSQL 16+ with the `pgvector` extension
- Docker (optional, for local Postgres)

### 1. Start PostgreSQL with pgvector

```bash
docker compose up -d
```

### 2. Configure

Copy the sample config and adjust the connection string:

```bash
cp config/sample.toml config/config.toml
```

The configuration file uses the following format:

```toml
database_connection_string = "postgres://myuser:mysecretpassword@127.0.0.1:5432/mydatabase"
storage_backend = "postgres"
```

### 3. Build and run

```bash
cargo build --release
cargo run --release
```

The server starts with both an MCP endpoint and an HTTP API.

## Tool Integration

LogAg acts as a **central memory store** across AI tools — Claude, Opencode, Codex, or any MCP/HTTP-compatible client. Record interactions from one tool and retrieve context from another.

Currently, Opencode support is merged in the repo. Other integrations follow the same pattern.

### Opencode Integration

[Opencode](https://opencode.ai) integration uses two MCP servers and an auto-recording plugin. The setup consists of two parts — recording sessions and retrieving cached responses.

#### Opencode Configuration

Register both MCP servers in `~/.config/opencode/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": ["/path/to/logag/AGENTS.md"],
  "mcp": {
    "logag": {
      "type": "remote",
      "url": "http://127.0.0.1:8000/mcp"
    },
    "logag-read": {
      "type": "remote",
      "url": "http://127.0.0.1:8000/read_mcp"
    }
  }
}
```

### AGENTS.md

Point to an `AGENTS.md` file that instructs the agent to check the cache on every query. This is referenced by the `instructions` field in your opencode config:

```
For every user query, first call `get_cached_agent_response` from the `logag-read`
MCP server to check if there is already cached knowledge around that prompt.
If cached content exists and is relevant, use it to inform your response.
```

### Plugin (Auto-Recording)

The [opencode plugin](js/opencode_plugin.js) hooks into chat events and automatically sends user/agent message pairs to the HTTP endpoint. Load it in your opencode config:

```json
{
  "plugins": ["/path/to/logag/js/opencode_plugin.js"]
}
```

The plugin listens for `chat.message` and `message.part.updated` events, buffers the conversation, and POSTs completed user/agent pairs to `http://localhost:8000/record`.

### Extending to Other Tools

To integrate another tool (Claude, Codex, etc.):

1. **Record** — send user/agent message pairs to `POST /record` (HTTP)
2. **Retrieve** — call the `logag-read` MCP tool `get_cached_agent_response` with the user's query text
3. **Configure** — register the MCP endpoints in that tool's MCP client config

### Data Flow

```
Opencode session
    │
    ├─ Plugin captures user input + agent output
    │  └─ POST /record ─→ LogAg stores as LogEvent + embedding
    │
    └─ Agent reads AGENTS.md
       └─ Calls logag-read_get_cached_agent_response(text)
          └─ MCP ─→ LogAg compares embedding via cosine distance
             └─ Returns cached agent output if similar query exists
```

## Usage

LogAg exposes two interfaces:

| Interface | Description |
|-----------|-------------|
| **MCP**   | Model Context Protocol — plug into any MCP-compatible AI client |
| **HTTP**  | REST API for custom integrations |

### MCP Tools

| Tool | Description |
|------|-------------|
| `logag_add_event` | Register a user/agent/thinking event - Not recommended since plugin based reads are more reliable and controllable |
| `logag-read_get_cached_agent_response` | Retrieve cached agent output for a similar user query |

## Configuration

| Key | Description | Default |
|-----|-------------|---------|
| `database_connection_string` | Postgres DSN with pgvector | — |
| `storage_backend` | Storage engine to use (`postgres`) | `postgres` |

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Open an issue** first to discuss the change you'd like to make.
2. **Branch from `main`** and submit a pull request.
3. **Keep commits small** and use conventional commit messages (e.g. `feat:`, `fix:`, `refactor:`).
4. **Run `cargo fmt`** and **`cargo clippy`** before pushing.
5. **Add tests** for new functionality. Integration tests requiring Postgres use `#[cfg(test)]` and a configurable connection string.
6. **Update `AGENTS.md`** if your change affects how the agent interacts with the codebase.

By contributing, you agree that your contributions will be licensed under the AGPL-3.0 license.

## License


See the [LICENSE](LICENSE) file for the full license text.
